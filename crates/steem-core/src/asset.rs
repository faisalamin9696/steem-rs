use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};

/// The asset symbols accepted by Steem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AssetSymbol {
    Steem,
    Sbd,
    Vests,
    Tests,
    Tbd,
}

impl AssetSymbol {
    pub const ALL: [AssetSymbol; 5] = [
        AssetSymbol::Steem,
        AssetSymbol::Sbd,
        AssetSymbol::Vests,
        AssetSymbol::Tests,
        AssetSymbol::Tbd,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            AssetSymbol::Steem => "STEEM",
            AssetSymbol::Sbd => "SBD",
            AssetSymbol::Vests => "VESTS",
            AssetSymbol::Tests => "TESTS",
            AssetSymbol::Tbd => "TBD",
        }
    }

    /// Number of decimal places used on the wire.
    pub fn precision(self) -> u8 {
        match self {
            AssetSymbol::Vests => 6,
            _ => 3,
        }
    }
}

impl fmt::Display for AssetSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AssetSymbol {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "STEEM" => Ok(AssetSymbol::Steem),
            "SBD" => Ok(AssetSymbol::Sbd),
            "VESTS" => Ok(AssetSymbol::Vests),
            "TESTS" => Ok(AssetSymbol::Tests),
            "TBD" => Ok(AssetSymbol::Tbd),
            other => Err(Error::InvalidAsset(format!(
                "unknown asset symbol {other:?}"
            ))),
        }
    }
}

impl Serialize for AssetSymbol {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AssetSymbol {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

/// A Steem asset amount stored in its smallest on-chain unit.
///
/// `1.234 STEEM` is stored as `1234`, and `1.000001 VESTS` as `1000001`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Asset {
    pub amount: i64,
    pub symbol: AssetSymbol,
}

impl Asset {
    pub const fn new(amount: i64, symbol: AssetSymbol) -> Self {
        Self { amount, symbol }
    }

    pub fn steem(amount: i64) -> Self {
        Self::new(amount, AssetSymbol::Steem)
    }

    pub fn sbd(amount: i64) -> Self {
        Self::new(amount, AssetSymbol::Sbd)
    }

    pub fn vests(amount: i64) -> Self {
        Self::new(amount, AssetSymbol::Vests)
    }

    pub fn precision(self) -> u8 {
        self.symbol.precision()
    }

    pub fn symbol(self) -> AssetSymbol {
        self.symbol
    }

    pub fn from_str_with_symbol(value: &str, expected: AssetSymbol) -> Result<Self> {
        let asset = Self::from_str(value)?;
        if asset.symbol != expected {
            return Err(Error::InvalidAsset(format!(
                "expected {}, got {}",
                expected, asset.symbol
            )));
        }
        Ok(asset)
    }

    pub fn parse(value: &str) -> Result<Self> {
        Self::from_str(value)
    }
}

impl FromStr for Asset {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts = s.split_whitespace();
        let amount = parts
            .next()
            .ok_or_else(|| Error::InvalidAsset(format!("missing amount in {s:?}")))?;
        let symbol = parts
            .next()
            .ok_or_else(|| Error::InvalidAsset(format!("missing symbol in {s:?}")))?;
        if parts.next().is_some() {
            return Err(Error::InvalidAsset(format!(
                "unexpected extra fields in {s:?}"
            )));
        }
        let symbol: AssetSymbol = symbol.parse()?;
        let precision = symbol.precision() as usize;

        let (negative, digits) = match amount.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, amount),
        };
        let (whole, fraction) = match digits.split_once('.') {
            Some((whole, fraction)) => (whole, fraction),
            None => (digits, ""),
        };
        if whole.is_empty() && fraction.is_empty() {
            return Err(Error::InvalidAsset(format!("empty amount in {s:?}")));
        }
        if fraction.len() > precision {
            return Err(Error::InvalidAsset(format!(
                "{} supports at most {precision} decimal places, got {}",
                symbol,
                fraction.len()
            )));
        }
        if !whole.chars().all(|c| c.is_ascii_digit())
            || !fraction.chars().all(|c| c.is_ascii_digit())
        {
            return Err(Error::InvalidAsset(format!("invalid amount {amount:?}")));
        }

        let whole: i128 = if whole.is_empty() {
            0
        } else {
            whole
                .parse()
                .map_err(|_| Error::InvalidAsset(format!("amount {amount:?} is too large")))?
        };
        let fraction_len = fraction.len();
        let fraction: i128 = if fraction.is_empty() {
            0
        } else {
            fraction
                .parse()
                .map_err(|_| Error::InvalidAsset(format!("invalid fraction in {amount:?}")))?
        };
        let scale = 10i128.pow(precision as u32);
        let fraction_scale = 10i128.pow((precision - fraction_len) as u32);
        let mut total = whole
            .checked_mul(scale)
            .and_then(|value| value.checked_add(fraction * fraction_scale))
            .ok_or_else(|| Error::InvalidAsset(format!("amount {amount:?} is too large")))?;
        if negative {
            total = -total;
        }
        let amount = i64::try_from(total)
            .map_err(|_| Error::InvalidAsset(format!("amount {s:?} is out of range")))?;
        Ok(Self { amount, symbol })
    }
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let precision = self.precision() as u32;
        let scale = 10i64.pow(precision);
        let negative = self.amount < 0;
        let absolute = self.amount.unsigned_abs();
        let whole = absolute / scale as u64;
        let fraction = absolute % scale as u64;
        let fraction_digits = if precision == 0 {
            String::new()
        } else {
            format!(".{:0width$}", fraction, width = precision as usize)
        };
        write!(
            f,
            "{}{}{} {}",
            if negative { "-" } else { "" },
            whole,
            fraction_digits,
            self.symbol
        )
    }
}

impl Serialize for Asset {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Asset {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_formats_steem() {
        let asset: Asset = "1.234 STEEM".parse().unwrap();
        assert_eq!(asset.amount, 1234);
        assert_eq!(asset.symbol, AssetSymbol::Steem);
        assert_eq!(asset.to_string(), "1.234 STEEM");
    }

    #[test]
    fn parses_and_formats_vests() {
        let asset: Asset = "12.112233 VESTS".parse().unwrap();
        assert_eq!(asset.amount, 12_112_233);
        assert_eq!(asset.to_string(), "12.112233 VESTS");
    }

    #[test]
    fn accepts_short_fractions() {
        assert_eq!("1 STEEM".parse::<Asset>().unwrap().amount, 1000);
        assert_eq!("1.2 STEEM".parse::<Asset>().unwrap().amount, 1200);
        assert_eq!("1.05 STEEM".parse::<Asset>().unwrap().amount, 1050);
        assert_eq!("1.050 STEEM".parse::<Asset>().unwrap().amount, 1050);
        assert_eq!("-0.001 STEEM".parse::<Asset>().unwrap().amount, -1);
    }

    #[test]
    fn rejects_too_many_decimals() {
        assert!("1.2345 STEEM".parse::<Asset>().is_err());
        assert!("1.1234567 VESTS".parse::<Asset>().is_err());
    }

    #[test]
    fn rejects_unknown_symbols() {
        assert!("1.000 DOGE".parse::<Asset>().is_err());
    }
}
