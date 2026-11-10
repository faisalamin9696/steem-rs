use thiserror::Error;

/// Errors produced by the Steem core crate.
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid asset: {0}")]
    InvalidAsset(String),

    #[error("invalid account name: {0}")]
    InvalidAccountName(String),

    #[error("invalid private key: {0}")]
    InvalidPrivateKey(String),

    #[error("invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("memo error: {0}")]
    Memo(String),

    #[error("unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("cryptographic error: {0}")]
    Crypto(String),

    #[error(transparent)]
    Secp256k1(#[from] secp256k1::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
