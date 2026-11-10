use std::fmt;
use std::str::FromStr;

use ripemd::Ripemd160;
use secp256k1::ecdsa::{RecoverableSignature, RecoveryId, Signature as SecpSignature};
use secp256k1::{Message, PublicKey as SecpPublicKey, Secp256k1, SecretKey};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::error::{Error, Result};

/// Steem's mainnet chain id. Testnets use a different id.
pub const MAINNET_CHAIN_ID: ChainId = ChainId([0u8; 32]);

/// A 32-byte chain id used to domain-separate transaction digests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChainId(pub [u8; 32]);

impl ChainId {
    pub const MAINNET: Self = MAINNET_CHAIN_ID;

    pub fn from_hex(value: &str) -> Result<Self> {
        let bytes = hex::decode(value)
            .map_err(|_| Error::Crypto("chain id is not valid hex".to_string()))?;
        if bytes.len() != 32 {
            return Err(Error::Crypto(format!(
                "chain id must be 32 bytes, got {}",
                bytes.len()
            )));
        }
        let mut id = [0u8; 32];
        id.copy_from_slice(&bytes);
        Ok(Self(id))
    }

    pub fn to_hex(self) -> String {
        hex::encode(self.0)
    }
}

impl Default for ChainId {
    fn default() -> Self {
        Self::MAINNET
    }
}

pub(crate) fn sha256(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

pub(crate) fn double_sha256(data: &[u8]) -> [u8; 32] {
    sha256(&sha256(data))
}

pub(crate) fn ripemd160(data: &[u8]) -> [u8; 20] {
    Ripemd160::digest(data).into()
}

/// A compressed secp256k1 public key encoded with Steem's `STM` prefix.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PublicKey {
    key: [u8; 33],
}

impl PublicKey {
    /// The all-zero key used by Steem for a disabled authority key.
    pub const fn null() -> Self {
        Self { key: [0u8; 33] }
    }

    pub fn from_bytes(bytes: [u8; 33]) -> Result<Self> {
        // Reject keys that are not a valid compressed point. The all-zero key
        // is allowed because Steem uses it for disabled authorities.
        if bytes == [0u8; 33] {
            return Ok(Self { key: bytes });
        }
        SecpPublicKey::from_slice(&bytes)?;
        Ok(Self { key: bytes })
    }

    pub fn as_bytes(&self) -> &[u8; 33] {
        &self.key
    }

    pub fn is_null(&self) -> bool {
        self.key == [0u8; 33]
    }

    pub(crate) fn from_secp(key: SecpPublicKey) -> Self {
        Self {
            key: key.serialize(),
        }
    }

    pub(crate) fn to_secp(self) -> Result<SecpPublicKey> {
        if self.is_null() {
            return Err(Error::InvalidPublicKey("null public key".to_string()));
        }
        Ok(SecpPublicKey::from_slice(&self.key)?)
    }

    pub fn to_string_with_prefix(self, prefix: &str) -> String {
        let mut payload = [0u8; 37];
        payload[..33].copy_from_slice(&self.key);
        let checksum = ripemd160(&self.key);
        payload[33..].copy_from_slice(&checksum[..4]);
        format!("{prefix}{}", bs58::encode(payload).into_string())
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_string_with_prefix("STM"))
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey({self})")
    }
}

impl FromStr for PublicKey {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        if value.len() < 4 {
            return Err(Error::InvalidPublicKey(
                "public key is too short".to_string(),
            ));
        }
        let prefix = value
            .get(..3)
            .ok_or_else(|| Error::InvalidPublicKey("invalid public key prefix".to_string()))?;
        if !prefix.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(Error::InvalidPublicKey(format!(
                "invalid public key prefix {prefix:?}"
            )));
        }
        let encoded = value
            .get(3..)
            .ok_or_else(|| Error::InvalidPublicKey("invalid public key body".to_string()))?;
        let decoded = bs58::decode(encoded)
            .into_vec()
            .map_err(|_| Error::InvalidPublicKey("public key is not valid base58".to_string()))?;
        if decoded.len() != 37 {
            return Err(Error::InvalidPublicKey(format!(
                "public key payload must be 37 bytes, got {}",
                decoded.len()
            )));
        }
        let mut key = [0u8; 33];
        key.copy_from_slice(&decoded[..33]);
        let checksum = ripemd160(&key);
        if decoded[33..] != checksum[..4] {
            return Err(Error::InvalidPublicKey(
                "public key checksum mismatch".to_string(),
            ));
        }
        Self::from_bytes(key)
    }
}

impl Serialize for PublicKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

/// A secp256k1 private key decoded from a Steem WIF string.
///
/// The secret is zeroized on drop and never implements `Debug` or `Serialize`.
pub struct PrivateKey {
    secret: Zeroizing<[u8; 32]>,
}

impl PrivateKey {
    pub fn from_wif(wif: &str) -> Result<Self> {
        let decoded = bs58::decode(wif)
            .into_vec()
            .map_err(|_| Error::InvalidPrivateKey("WIF is not valid base58".to_string()))?;
        if decoded.len() != 37 {
            return Err(Error::InvalidPrivateKey(format!(
                "WIF payload must be 37 bytes, got {}",
                decoded.len()
            )));
        }
        if decoded[0] != 0x80 {
            return Err(Error::InvalidPrivateKey(
                "WIF network prefix must be 0x80".to_string(),
            ));
        }
        let checksum = double_sha256(&decoded[..33]);
        if decoded[33..] != checksum[..4] {
            return Err(Error::InvalidPrivateKey(
                "WIF checksum mismatch".to_string(),
            ));
        }
        let mut secret = Zeroizing::new([0u8; 32]);
        secret.copy_from_slice(&decoded[1..33]);
        Ok(Self { secret })
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Result<Self> {
        SecretKey::from_slice(&bytes)?;
        Ok(Self {
            secret: Zeroizing::new(bytes),
        })
    }

    /// Expose the WIF encoding. Treat the result as a secret.
    pub fn to_wif(&self) -> String {
        let mut payload = [0u8; 33];
        payload[0] = 0x80;
        payload[1..].copy_from_slice(&self.secret[..]);
        let checksum = double_sha256(&payload);
        let mut encoded = [0u8; 37];
        encoded[..33].copy_from_slice(&payload);
        encoded[33..].copy_from_slice(&checksum[..4]);
        bs58::encode(encoded).into_string()
    }

    pub fn public_key(&self) -> Result<PublicKey> {
        let secret = self.to_secp_secret()?;
        let secp = Secp256k1::signing_only();
        Ok(PublicKey::from_secp(SecpPublicKey::from_secret_key(
            &secp, &secret,
        )))
    }

    pub fn sign_digest(&self, digest: &[u8; 32]) -> Result<Signature> {
        let secret = self.to_secp_secret()?;
        let secp = Secp256k1::new();
        let message = Message::from_digest_slice(digest)?;

        // Steem requires a canonical compact signature. Vary the RFC6979
        // nonce data until the r/s high-bit rules are satisfied.
        for counter in 0u32..=255 {
            let mut hasher = Sha256::new();
            hasher.update(digest);
            hasher.update(counter.to_le_bytes());
            let noncedata: [u8; 32] = hasher.finalize().into();
            let recoverable =
                secp.sign_ecdsa_recoverable_with_noncedata(&message, &secret, &noncedata);
            let (recovery, compact) = recoverable.serialize_compact();
            if is_canonical_signature(&compact) {
                let mut bytes = [0u8; 65];
                bytes[0] = 31 + recovery.to_i32() as u8;
                bytes[1..].copy_from_slice(&compact);
                return Ok(Signature { bytes });
            }
        }
        Err(Error::Crypto(
            "unable to produce a canonical signature".to_string(),
        ))
    }

    pub(crate) fn to_secp_secret(&self) -> Result<SecretKey> {
        Ok(SecretKey::from_slice(&self.secret[..])?)
    }
}

impl Clone for PrivateKey {
    fn clone(&self) -> Self {
        Self {
            secret: Zeroizing::new(*self.secret),
        }
    }
}

impl PartialEq for PrivateKey {
    fn eq(&self, other: &Self) -> bool {
        self.secret[..] == other.secret[..]
    }
}

impl Eq for PrivateKey {}

impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PrivateKey(<redacted>)")
    }
}

/// A 65-byte Steem compact signature: `header || r || s`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Signature {
    bytes: [u8; 65],
}

impl Signature {
    pub fn from_bytes(bytes: [u8; 65]) -> Result<Self> {
        let header = bytes[0] as i32 - 31;
        if !(0..=3).contains(&header) {
            return Err(Error::InvalidSignature(format!(
                "signature header {} is out of range",
                bytes[0]
            )));
        }
        Ok(Self { bytes })
    }

    pub fn from_hex(value: &str) -> Result<Self> {
        let bytes = hex::decode(value)
            .map_err(|_| Error::InvalidSignature("signature is not valid hex".to_string()))?;
        if bytes.len() != 65 {
            return Err(Error::InvalidSignature(format!(
                "signature must be 65 bytes, got {}",
                bytes.len()
            )));
        }
        let mut out = [0u8; 65];
        out.copy_from_slice(&bytes);
        Self::from_bytes(out)
    }

    pub fn to_hex(self) -> String {
        hex::encode(self.bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 65] {
        &self.bytes
    }

    pub fn recover(self, digest: &[u8; 32]) -> Result<PublicKey> {
        let recovery = RecoveryId::from_i32(self.bytes[0] as i32 - 31)?;
        let compact: [u8; 64] = self.bytes[1..].try_into().unwrap();
        let signature = RecoverableSignature::from_compact(&compact, recovery)?;
        let message = Message::from_digest_slice(digest)?;
        let secp = Secp256k1::new();
        Ok(PublicKey::from_secp(
            secp.recover_ecdsa(&message, &signature)?,
        ))
    }

    pub fn verify(self, digest: &[u8; 32], public_key: PublicKey) -> Result<()> {
        let compact: [u8; 64] = self.bytes[1..].try_into().unwrap();
        let signature = SecpSignature::from_compact(&compact)?;
        let message = Message::from_digest_slice(digest)?;
        let secp = Secp256k1::verification_only();
        Ok(secp.verify_ecdsa(&message, &signature, &public_key.to_secp()?)?)
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({})", self.to_hex())
    }
}

impl Serialize for Signature {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::from_hex(&value).map_err(serde::de::Error::custom)
    }
}

/// SHA-256(chain_id || serialized_transaction).
pub fn transaction_digest(transaction_bytes: &[u8], chain_id: &ChainId) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(chain_id.0);
    hasher.update(transaction_bytes);
    hasher.finalize().into()
}

/// Steem's canonical compact-signature rule.
fn is_canonical_signature(signature: &[u8; 64]) -> bool {
    !(signature[0] & 0x80 != 0)
        && !(signature[0] == 0 && (signature[1] & 0x80) == 0)
        && !(signature[32] & 0x80 != 0)
        && !(signature[32] == 0 && (signature[33] & 0x80) == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIF: &str = "5KBVDxspWUB59evtsGy19euUbV2HmujS6DUMHU62XUMsDCoPdWS";
    const PUBLIC: &str = "STM4zupkLHXpMLBmQEWc2sPBuGcdm2dJb4GNNvaDkVbr1Zo3GNFmC";

    #[test]
    fn wif_round_trip() {
        let key = PrivateKey::from_wif(WIF).unwrap();
        assert_eq!(key.to_wif(), WIF);
        assert_eq!(key.public_key().unwrap().to_string(), PUBLIC);
    }

    #[test]
    fn public_key_round_trip() {
        let key: PublicKey = PUBLIC.parse().unwrap();
        assert_eq!(key.to_string(), PUBLIC);
    }

    #[test]
    fn signs_and_recovers() {
        let key = PrivateKey::from_wif(WIF).unwrap();
        let digest = sha256(b"steem-core-test");
        let signature = key.sign_digest(&digest).unwrap();
        let recovered = signature.recover(&digest).unwrap();
        assert_eq!(recovered, key.public_key().unwrap());
        signature.verify(&digest, recovered).unwrap();
    }

    #[test]
    fn rejects_bad_checksum() {
        let mut wif = WIF.to_string();
        wif.pop();
        wif.push('1');
        assert!(PrivateKey::from_wif(&wif).is_err());
    }
}
