#![forbid(unsafe_code)]

//! Core Steem protocol primitives.
//!
//! This crate is intentionally transport-free: it models assets, account names,
//! authorities, operations, transactions, signing and memos. The JSON-RPC
//! transport lives in `steem-rpc`.
//!
//! Security notes:
//! * [`PrivateKey`] zeroizes its secret on drop and does not implement
//!   `Debug` or `Serialize`.
//! * Signing uses libsecp256k1 with RFC6979 nonces and Steem's canonical
//!   signature rule.
//! * Memo encryption mirrors dsteem's ECDH + SHA-512 + AES-256-CBC format.

pub mod account;
pub mod asset;
pub mod authority;
pub mod codec;
pub mod crypto;
pub mod error;
pub mod memo;
pub mod operation;
pub mod transaction;

pub use account::AccountName;
pub use asset::{Asset, AssetSymbol};
pub use authority::Authority;
pub use codec::{Decoder, Encoder};
pub use crypto::{transaction_digest, ChainId, PrivateKey, PublicKey, Signature, MAINNET_CHAIN_ID};
pub use error::{Error, Result};
pub use memo::Memo;
pub use operation::ChainProperties;
pub use operation::Operation;
pub use transaction::{expiration_from_chain_time, transaction_id, SignedTransaction, Transaction};
