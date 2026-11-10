#![forbid(unsafe_code)]

//! A small blocking JSON-RPC client for Steem nodes.
//!
//! The client never sees private keys: it only accepts already-signed
//! transactions. That keeps the signing path in `steem-core` auditable.

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use thiserror::Error;

use steem_core::{ChainId, SignedTransaction};

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Steem RPC error {code}: {message}")]
    Rpc { code: i64, message: String },

    #[error("invalid RPC response: {0}")]
    InvalidResponse(String),
}

/// Dynamic global properties returned by `condenser_api.get_dynamic_global_properties`.
#[derive(Clone, Debug, Deserialize)]
pub struct DynamicGlobalProperties {
    pub head_block_number: u32,
    pub head_block_id: String,
    pub time: String,
    #[serde(default)]
    pub virtual_supply: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct Version {
    chain_id: String,
}

/// A blocking Steem JSON-RPC client.
#[derive(Clone, Debug)]
pub struct RpcClient {
    url: String,
    http: reqwest::blocking::Client,
}

impl RpcClient {
    pub fn new(url: impl Into<String>) -> Result<Self, RpcError> {
        let http = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        Ok(Self {
            url: url.into(),
            http,
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    /// Call any JSON-RPC method and deserialize its `result`.
    pub fn call<T: DeserializeOwned>(&self, method: &str, params: Value) -> Result<T, RpcError> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        let response = self.http.post(&self.url).json(&body).send()?;
        let status = response.status();
        let text = response.text()?;
        if !status.is_success() {
            let preview: String = text.chars().take(200).collect();
            return Err(RpcError::InvalidResponse(format!(
                "HTTP {status}: {preview}"
            )));
        }
        let value: Value = serde_json::from_str(&text)?;
        if let Some(error) = value.get("error") {
            let code = error.get("code").and_then(Value::as_i64).unwrap_or(0);
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("unknown RPC error")
                .to_string();
            return Err(RpcError::Rpc { code, message });
        }
        let result = value
            .get("result")
            .cloned()
            .ok_or_else(|| RpcError::InvalidResponse("response has no result".to_string()))?;
        Ok(serde_json::from_value(result)?)
    }

    pub fn get_version(&self) -> Result<Value, RpcError> {
        self.call("condenser_api.get_version", json!([]))
    }

    pub fn get_chain_id(&self) -> Result<ChainId, RpcError> {
        let version: Version = self.call("condenser_api.get_version", json!([]))?;
        ChainId::from_hex(&version.chain_id)
            .map_err(|error| RpcError::InvalidResponse(error.to_string()))
    }

    pub fn get_dynamic_global_properties(&self) -> Result<DynamicGlobalProperties, RpcError> {
        self.call("condenser_api.get_dynamic_global_properties", json!([]))
    }

    pub fn get_accounts(&self, accounts: &[&str]) -> Result<Vec<Value>, RpcError> {
        self.call("condenser_api.get_accounts", json!([accounts]))
    }

    /// Broadcast a signed transaction and return the node's response.
    pub fn broadcast_transaction(
        &self,
        transaction: &SignedTransaction,
    ) -> Result<Value, RpcError> {
        self.call(
            "condenser_api.broadcast_transaction",
            json!([transaction.to_json_value()]),
        )
    }

    /// Broadcast and wait for the transaction to be included in a block.
    pub fn broadcast_transaction_synchronous(
        &self,
        transaction: &SignedTransaction,
    ) -> Result<Value, RpcError> {
        self.call(
            "condenser_api.broadcast_transaction_synchronous",
            json!([transaction.to_json_value()]),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Read-only smoke test against a public node. Run with:
    /// `cargo test -p steem-rpc -- --ignored`
    #[test]
    #[ignore]
    fn live_node_smoke_test() {
        let client = RpcClient::new("https://api.steemit.com").expect("client");
        let props = client
            .get_dynamic_global_properties()
            .expect("dynamic global properties");
        assert!(props.head_block_number > 0);
        assert!(props.head_block_id.len() >= 16);

        let accounts = client.get_accounts(&["steemit"]).expect("accounts");
        assert!(!accounts.is_empty());
    }

    /// Build and sign a transaction from the live head block. Read-only: it
    /// never broadcasts.
    #[test]
    #[ignore]
    fn live_node_build_and_sign() {
        use steem_core::{Asset, ChainId, Operation, PrivateKey, Transaction};

        let client = RpcClient::new("https://api.steemit.com").expect("client");
        let props = client
            .get_dynamic_global_properties()
            .expect("dynamic global properties");
        let key = PrivateKey::from_wif("5KBVDxspWUB59evtsGy19euUbV2HmujS6DUMHU62XUMsDCoPdWS")
            .expect("key");
        let transaction = Transaction::from_chain(
            props.head_block_number,
            &props.head_block_id,
            &props.time,
            1,
            vec![Operation::Transfer {
                from: "alice".into(),
                to: "bob".into(),
                amount: "1.000 STEEM".parse::<Asset>().expect("asset"),
                memo: "smoke test".into(),
            }],
        )
        .expect("transaction");
        let signed = transaction.sign(&key, &ChainId::MAINNET).expect("sign");
        assert_eq!(
            signed
                .recover_public_keys(&ChainId::MAINNET)
                .expect("recover"),
            vec![key.public_key().expect("public key")]
        );
    }
}
