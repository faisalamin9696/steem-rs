use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};

/// A validated Steem account name.
///
/// Steem account names are 3-16 characters, start with a lowercase letter, and
/// may contain lowercase letters, digits, `-` and `.`. They may not end with
/// `-` or `.`, and may not contain consecutive `-` or `..`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AccountName(String);

impl AccountName {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        Self::validate(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn validate(value: &str) -> Result<()> {
        let bytes = value.as_bytes();
        if bytes.len() < 3 || bytes.len() > 16 {
            return Err(Error::InvalidAccountName(format!(
                "account names must be 3-16 characters, got {}",
                bytes.len()
            )));
        }
        if !bytes[0].is_ascii_lowercase() {
            return Err(Error::InvalidAccountName(
                "account names must start with a lowercase letter".to_string(),
            ));
        }
        for &byte in bytes {
            let valid =
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'.';
            if !valid {
                return Err(Error::InvalidAccountName(format!(
                    "invalid character {:?}",
                    byte as char
                )));
            }
        }
        if matches!(bytes.last(), Some(b'-') | Some(b'.')) {
            return Err(Error::InvalidAccountName(
                "account names may not end with '-' or '.'".to_string(),
            ));
        }
        if value.contains("--") || value.contains("..") {
            return Err(Error::InvalidAccountName(
                "account names may not contain '--' or '..'".to_string(),
            ));
        }
        Ok(())
    }
}

impl fmt::Display for AccountName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for AccountName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for AccountName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::new(s)
    }
}

impl TryFrom<String> for AccountName {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<&str> for AccountName {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl From<AccountName> for String {
    fn from(value: AccountName) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_names() {
        for name in ["alice", "steem-vault", "a.b-c9", "abc"] {
            assert!(AccountName::new(name).is_ok(), "{name} should be valid");
        }
    }

    #[test]
    fn rejects_invalid_names() {
        for name in [
            "",
            "ab",
            "Alice",
            "1alice",
            "-alice",
            "alice-",
            "alice.",
            "a--b",
            "a..b",
            "alice_",
            "averylongaccountname",
        ] {
            assert!(AccountName::new(name).is_err(), "{name} should be invalid");
        }
    }
}
