use serde_json::{json, Value};

use crate::codec::Encoder;
use crate::crypto::{transaction_digest, ChainId, PrivateKey, PublicKey, Signature};
use crate::error::{Error, Result};
use crate::operation::Operation;

/// A Steem transaction before signatures are attached.
#[derive(Clone, Debug, PartialEq)]
pub struct Transaction {
    pub ref_block_num: u16,
    pub ref_block_prefix: u32,
    /// UTC timestamp without a timezone suffix, e.g. `2026-11-10T12:00:00`.
    pub expiration: String,
    pub operations: Vec<Operation>,
    pub extensions: Vec<String>,
}

impl Transaction {
    pub fn new(
        ref_block_num: u16,
        ref_block_prefix: u32,
        expiration: impl Into<String>,
        operations: Vec<Operation>,
    ) -> Self {
        Self {
            ref_block_num,
            ref_block_prefix,
            expiration: expiration.into(),
            operations,
            extensions: Vec::new(),
        }
    }

    /// Build a transaction from the node's head block and chain time.
    pub fn from_chain(
        head_block_number: u32,
        head_block_id: &str,
        chain_time: &str,
        expiration_minutes: u32,
        operations: Vec<Operation>,
    ) -> Result<Self> {
        Ok(Self {
            ref_block_num: (head_block_number & 0xffff) as u16,
            ref_block_prefix: parse_ref_block_prefix(head_block_id)?,
            expiration: expiration_from_chain_time(chain_time, expiration_minutes)?,
            operations,
            extensions: Vec::new(),
        })
    }

    /// Parse the JSON shape used by the wallet and by Steem's broadcast API.
    pub fn from_json_value(value: &Value) -> Result<Self> {
        let object = value.as_object().ok_or_else(|| {
            Error::InvalidTransaction("transaction must be an object".to_string())
        })?;
        let ref_block_num = object
            .get("ref_block_num")
            .and_then(Value::as_u64)
            .ok_or_else(|| Error::InvalidTransaction("missing ref_block_num".to_string()))?;
        let ref_block_num = u16::try_from(ref_block_num)
            .map_err(|_| Error::InvalidTransaction("ref_block_num is out of range".to_string()))?;
        let ref_block_prefix = object
            .get("ref_block_prefix")
            .and_then(Value::as_u64)
            .ok_or_else(|| Error::InvalidTransaction("missing ref_block_prefix".to_string()))?;
        let ref_block_prefix = u32::try_from(ref_block_prefix).map_err(|_| {
            Error::InvalidTransaction("ref_block_prefix is out of range".to_string())
        })?;
        let expiration = object
            .get("expiration")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::InvalidTransaction("missing expiration".to_string()))?
            .to_string();
        let operations = object
            .get("operations")
            .and_then(Value::as_array)
            .ok_or_else(|| Error::InvalidTransaction("missing operations".to_string()))?
            .iter()
            .map(Operation::from_tuple)
            .collect::<Result<Vec<_>>>()?;
        let extensions = object
            .get("extensions")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            ref_block_num,
            ref_block_prefix,
            expiration,
            operations,
            extensions,
        })
    }

    pub fn expiration_timestamp(&self) -> Result<u32> {
        let trimmed = self.expiration.trim_end_matches('Z');
        let parsed = chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S")
            .or_else(|_| chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S%.f"))
            .map_err(|_| {
                Error::InvalidTransaction(format!(
                    "expiration {:?} must be YYYY-MM-DDTHH:MM:SS",
                    self.expiration
                ))
            })?;
        let timestamp = parsed.and_utc().timestamp();
        u32::try_from(timestamp).map_err(|_| {
            Error::InvalidTransaction("expiration is outside the uint32 range".to_string())
        })
    }

    /// Serialize the transaction using Steem's Graphene wire format.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut encoder = Encoder::new();
        encoder.write_u16_le(self.ref_block_num);
        encoder.write_u32_le(self.ref_block_prefix);
        encoder.write_u32_le(self.expiration_timestamp()?);
        encoder.write_varint32(self.operations.len() as u32);
        for operation in &self.operations {
            operation.serialize(&mut encoder);
        }
        encoder.write_varint32(self.extensions.len() as u32);
        for extension in &self.extensions {
            encoder.write_vstring(extension);
        }
        Ok(encoder.into_bytes())
    }

    pub fn digest(&self, chain_id: &ChainId) -> Result<[u8; 32]> {
        Ok(transaction_digest(&self.serialize()?, chain_id))
    }

    pub fn sign(&self, key: &PrivateKey, chain_id: &ChainId) -> Result<SignedTransaction> {
        let digest = self.digest(chain_id)?;
        let signature = key.sign_digest(&digest)?;
        Ok(SignedTransaction {
            transaction: self.clone(),
            signatures: vec![signature],
        })
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        json!({
            "ref_block_num": self.ref_block_num,
            "ref_block_prefix": self.ref_block_prefix,
            "expiration": self.expiration,
            "operations": self
                .operations
                .iter()
                .map(Operation::to_tuple)
                .collect::<Vec<_>>(),
            "extensions": self.extensions,
        })
    }
}

/// A transaction with one or more compact signatures.
#[derive(Clone, Debug, PartialEq)]
pub struct SignedTransaction {
    pub transaction: Transaction,
    pub signatures: Vec<Signature>,
}

impl SignedTransaction {
    pub fn digest(&self, chain_id: &ChainId) -> Result<[u8; 32]> {
        self.transaction.digest(chain_id)
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        let mut value = self.transaction.to_json_value();
        value["signatures"] = json!(self
            .signatures
            .iter()
            .map(|signature| signature.to_hex())
            .collect::<Vec<_>>());
        value
    }

    /// Recover the public key for every signature over this transaction.
    pub fn recover_public_keys(&self, chain_id: &ChainId) -> Result<Vec<PublicKey>> {
        let digest = self.digest(chain_id)?;
        self.signatures
            .iter()
            .map(|signature| signature.recover(&digest))
            .collect()
    }
}

/// Add `minutes` to a chain time string (`YYYY-MM-DDTHH:MM:SS`).
pub fn expiration_from_chain_time(chain_time: &str, minutes: u32) -> Result<String> {
    let trimmed = chain_time.trim_end_matches('Z');
    let parsed = chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S%.f"))
        .map_err(|_| Error::InvalidTransaction(format!("invalid chain time {chain_time:?}")))?;
    let expiration = parsed + chrono::Duration::minutes(minutes as i64);
    Ok(expiration.format("%Y-%m-%dT%H:%M:%S").to_string())
}

fn parse_ref_block_prefix(head_block_id: &str) -> Result<u32> {
    let bytes = hex::decode(head_block_id)
        .map_err(|_| Error::InvalidTransaction("invalid head block id".to_string()))?;
    if bytes.len() < 8 {
        return Err(Error::InvalidTransaction(
            "head block id is too short".to_string(),
        ));
    }
    Ok(u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]))
}

/// dsteem-compatible transaction id: first 20 bytes of SHA-256(serialized tx).
pub fn transaction_id(transaction_bytes: &[u8]) -> String {
    hex::encode(crate::crypto::sha256(transaction_bytes))[..40].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::Asset;

    fn vector_transaction() -> Transaction {
        Transaction::new(
            1234,
            567_890_123,
            "2026-11-10T12:00:00",
            vec![Operation::Transfer {
                from: "alice".to_string(),
                to: "bob".to_string(),
                amount: "1.234 STEEM".parse::<Asset>().unwrap(),
                memo: "hello".to_string(),
            }],
        )
    }

    #[test]
    fn serializes_the_dsteem_vector() {
        let transaction = vector_transaction();
        assert_eq!(
            hex::encode(transaction.serialize().unwrap()),
            "d204cb50d9214007f36a010205616c69636503626f62d20400000000000003535445454d00000568656c6c6f00"
        );
    }

    #[test]
    fn digest_matches_dsteem() {
        let transaction = vector_transaction();
        assert_eq!(
            hex::encode(transaction.digest(&ChainId::MAINNET).unwrap()),
            "f34cede61c92ea5431c7f90e0ed5c52d97f8d8ba8a4ccc7992e26d4abfb076d2"
        );
    }

    #[test]
    fn json_round_trip_preserves_serialization() {
        let transaction = vector_transaction();
        let json = transaction.to_json_value();
        let parsed = Transaction::from_json_value(&json).expect("parse transaction");
        assert_eq!(
            parsed.serialize().unwrap(),
            transaction.serialize().unwrap()
        );
    }

    #[test]
    fn signs_and_recovers_the_sender() {
        let key =
            PrivateKey::from_wif("5KBVDxspWUB59evtsGy19euUbV2HmujS6DUMHU62XUMsDCoPdWS").unwrap();
        let signed = vector_transaction().sign(&key, &ChainId::MAINNET).unwrap();
        let recovered = signed.recover_public_keys(&ChainId::MAINNET).unwrap();
        assert_eq!(recovered, vec![key.public_key().unwrap()]);
    }
}
