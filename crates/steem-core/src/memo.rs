use aes::Aes256;
use cbc::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rand::Rng;
use secp256k1::ecdh::shared_secret_point;
use sha2::{Digest, Sha256, Sha512};

use crate::codec::{Decoder, Encoder};
use crate::crypto::{PrivateKey, PublicKey};
use crate::error::{Error, Result};

type Aes256CbcEnc = cbc::Encryptor<Aes256>;
type Aes256CbcDec = cbc::Decryptor<Aes256>;

/// Steem memo encryption compatible with dsteem's `Memo.encode`/`Memo.decode`.
pub struct Memo;

impl Memo {
    /// Encrypt a plain memo. The result is `#` followed by base58 ciphertext.
    pub fn encrypt(
        private_key: &PrivateKey,
        recipient: &PublicKey,
        memo: &str,
        nonce: u64,
    ) -> Result<String> {
        if recipient.is_null() {
            return Err(Error::Memo("recipient has no memo public key".to_string()));
        }
        let shared = shared_secret(private_key, recipient)?;
        let key = encryption_key(&shared, nonce);
        let aes_key = &key[..32];
        let iv = &key[32..48];

        // dsteem treats the leading `#` as the "encrypted memo" marker and
        // strips it before writing the plaintext into the ciphertext.
        let body = memo.strip_prefix('#').unwrap_or(memo);
        let mut plaintext = Encoder::new();
        plaintext.write_vstring(body);
        let plaintext = plaintext.into_bytes();

        let encryptor = Aes256CbcEnc::new_from_slices(aes_key, iv)
            .map_err(|_| Error::Memo("invalid AES key or IV".to_string()))?;
        let ciphertext = encryptor.encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

        let check = checksum(&key);
        let mut encoded = Encoder::new();
        encoded.write_bytes(private_key.public_key()?.as_bytes());
        encoded.write_bytes(recipient.as_bytes());
        encoded.write_u64_le(nonce);
        encoded.write_u32_le(check);
        encoded.write_vbytes(&ciphertext);

        Ok(format!(
            "#{}",
            bs58::encode(encoded.into_bytes()).into_string()
        ))
    }

    /// Decrypt a `#`-prefixed memo. The returned string keeps the leading `#`.
    pub fn decrypt(private_key: &PrivateKey, memo: &str) -> Result<String> {
        let encoded = memo
            .strip_prefix('#')
            .ok_or_else(|| Error::Memo("memo is not encrypted".to_string()))?;
        let bytes = bs58::decode(encoded)
            .into_vec()
            .map_err(|_| Error::Memo("memo is not valid base58".to_string()))?;

        let mut decoder = Decoder::new(&bytes);
        let from = read_public_key(&mut decoder)?;
        let to = read_public_key(&mut decoder)?;
        let nonce = decoder.read_u64_le()?;
        let expected_check = decoder.read_u32_le()?;
        let ciphertext = decoder.read_vbytes()?.to_vec();

        let our_public = private_key.public_key()?;
        let other = if our_public == from { to } else { from };
        let shared = shared_secret(private_key, &other)?;
        let key = encryption_key(&shared, nonce);
        if checksum(&key) != expected_check {
            return Err(Error::Memo("memo checksum mismatch".to_string()));
        }

        let aes_key = &key[..32];
        let iv = &key[32..48];
        let decryptor = Aes256CbcDec::new_from_slices(aes_key, iv)
            .map_err(|_| Error::Memo("invalid AES key or IV".to_string()))?;
        let plaintext = decryptor
            .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
            .map_err(|_| Error::Memo("memo could not be decrypted".to_string()))?;

        // Newer memos are a vstring; older clients wrote the raw memo bytes.
        let mut decoder = Decoder::new(&plaintext);
        let text = match decoder.read_vstring() {
            Ok(text) if decoder.remaining() == 0 => text,
            _ => String::from_utf8(plaintext)
                .map_err(|_| Error::Memo("memo is not valid UTF-8".to_string()))?,
        };
        Ok(format!("#{text}"))
    }

    /// Generate a random 64-bit memo nonce. The nonce is public on-chain.
    pub fn random_nonce() -> u64 {
        rand::thread_rng().gen()
    }
}

fn read_public_key(decoder: &mut Decoder<'_>) -> Result<PublicKey> {
    let bytes: [u8; 33] = decoder
        .read_bytes(33)?
        .try_into()
        .map_err(|_| Error::Memo("truncated public key".to_string()))?;
    PublicKey::from_bytes(bytes)
}

fn shared_secret(private_key: &PrivateKey, public_key: &PublicKey) -> Result<[u8; 64]> {
    let point = shared_secret_point(&public_key.to_secp()?, &private_key.to_secp_secret()?);
    let mut hasher = Sha512::new();
    hasher.update(&point[..32]);
    Ok(hasher.finalize().into())
}

fn encryption_key(shared: &[u8; 64], nonce: u64) -> [u8; 64] {
    let mut hasher = Sha512::new();
    hasher.update(nonce.to_le_bytes());
    hasher.update(shared);
    hasher.finalize().into()
}

fn checksum(key: &[u8; 64]) -> u32 {
    let digest = Sha256::digest(key);
    u32::from_le_bytes(digest[..4].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENDER_WIF: &str = "5KBVDxspWUB59evtsGy19euUbV2HmujS6DUMHU62XUMsDCoPdWS";
    const RECIPIENT_WIF: &str = "5JokHYyDCesmw6nWgQH97nwUk548FK2DgWT3kv7PGsyRBHytqgp";
    const RECIPIENT_PUB: &str = "STM78ZtLEGQ8aWPDcywQQNSRyBmTy1ZVfs42c3F9BWBivasfgfkUR";
    const CIPHERTEXT: &str = "#AK3RakcoqgZEFczbnGS68reBXYpDNs5pQdJ24g1ZPtoyb6HaPyRSpD3885r5gbSBGHW1f5UmHKseLSqAg4efLVDzPq2NMuZW366zG2jvn54t9eoYriFm5Mqpui1Gg7dQy";
    const NONCE: u64 = 123_456_789_012_345_678;

    #[test]
    fn decrypts_a_dsteem_memo() {
        let recipient = PrivateKey::from_wif(RECIPIENT_WIF).unwrap();
        assert_eq!(
            Memo::decrypt(&recipient, CIPHERTEXT).unwrap(),
            "#hello memo"
        );
    }

    #[test]
    fn encrypts_the_same_bytes_as_dsteem() {
        let sender = PrivateKey::from_wif(SENDER_WIF).unwrap();
        let recipient: PublicKey = RECIPIENT_PUB.parse().unwrap();
        let encrypted = Memo::encrypt(&sender, &recipient, "#hello memo", NONCE).unwrap();
        assert_eq!(encrypted, CIPHERTEXT);
    }

    #[test]
    fn round_trips() {
        let sender = PrivateKey::from_wif(SENDER_WIF).unwrap();
        let recipient = PrivateKey::from_wif(RECIPIENT_WIF).unwrap();
        let encrypted = Memo::encrypt(
            &sender,
            &recipient.public_key().unwrap(),
            "round trip",
            Memo::random_nonce(),
        )
        .unwrap();
        assert_eq!(
            Memo::decrypt(&recipient, &encrypted).unwrap(),
            "#round trip"
        );
    }
}
