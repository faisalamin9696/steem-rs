#![forbid(unsafe_code)]

//! Umbrella crate for the Rust Steem SDK.
//!
//! ```no_run
//! use steem::{Asset, ChainId, Operation, PrivateKey, Transaction};
//!
//! # fn main() -> steem::Result<()> {
//! let key = PrivateKey::from_wif("5K...")?;
//! let tx = Transaction::new(
//!     1234,
//!     567_890_123,
//!     "2026-11-10T12:00:00",
//!     vec![Operation::Transfer {
//!         from: "alice".into(),
//!         to: "bob".into(),
//!         amount: "1.000 STEEM".parse::<Asset>()?,
//!         memo: "hello".into(),
//!     }],
//! );
//! let signed = tx.sign(&key, &ChainId::MAINNET)?;
//! # Ok(())
//! # }
//! ```

pub use steem_core::*;

#[cfg(feature = "rpc")]
pub use steem_rpc::*;
