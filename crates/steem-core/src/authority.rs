use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::codec::Encoder;
use crate::crypto::PublicKey;

/// A Steem authority: a weighted set of keys and accounts with a threshold.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Authority {
    pub weight_threshold: u32,
    pub account_auths: Vec<(String, u16)>,
    pub key_auths: Vec<(PublicKey, u16)>,
}

impl Authority {
    pub fn new(weight_threshold: u32) -> Self {
        Self {
            weight_threshold,
            account_auths: Vec::new(),
            key_auths: Vec::new(),
        }
    }

    pub fn add_key(mut self, key: PublicKey, weight: u16) -> Self {
        self.key_auths.push((key, weight));
        self
    }

    pub fn add_account(mut self, account: impl Into<String>, weight: u16) -> Self {
        self.account_auths.push((account.into(), weight));
        self
    }

    /// Sum the weights of keys present in `keys`.
    pub fn weight_of(&self, keys: &HashSet<PublicKey>) -> u32 {
        self.key_auths
            .iter()
            .filter(|(key, _)| keys.contains(key))
            .map(|(_, weight)| *weight as u32)
            .sum()
    }

    /// True when the supplied keys meet this authority's threshold.
    pub fn is_satisfied(&self, keys: &HashSet<PublicKey>) -> bool {
        self.weight_of(keys) >= self.weight_threshold
    }

    /// Evaluate nested `account_auths` recursively.
    ///
    /// `resolve` returns the authority of a named account. The recursion is
    /// bounded to avoid cycles in malformed authority graphs.
    pub fn is_satisfied_with_accounts<F>(&self, keys: &HashSet<PublicKey>, resolve: &F) -> bool
    where
        F: Fn(&str) -> Option<Authority> + ?Sized,
    {
        self.satisfied_depth(keys, resolve, 0)
    }

    fn satisfied_depth<F>(&self, keys: &HashSet<PublicKey>, resolve: &F, depth: usize) -> bool
    where
        F: Fn(&str) -> Option<Authority> + ?Sized,
    {
        if depth > 4 {
            return false;
        }
        let mut weight = self.weight_of(keys);
        for (account, account_weight) in &self.account_auths {
            if let Some(authority) = resolve(account) {
                if authority.satisfied_depth(keys, resolve, depth + 1) {
                    weight += *account_weight as u32;
                }
            }
        }
        weight >= self.weight_threshold
    }

    pub(crate) fn serialize(&self, encoder: &mut Encoder) {
        encoder.write_u32_le(self.weight_threshold);

        encoder.write_varint32(self.account_auths.len() as u32);
        for (account, weight) in &self.account_auths {
            encoder.write_vstring(account);
            encoder.write_u16_le(*weight);
        }

        encoder.write_varint32(self.key_auths.len() as u32);
        for (key, weight) in &self.key_auths {
            encoder.write_bytes(key.as_bytes());
            encoder.write_u16_le(*weight);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weights_keys_and_accounts() {
        let key = "STM4zupkLHXpMLBmQEWc2sPBuGcdm2dJb4GNNvaDkVbr1Zo3GNFmC"
            .parse::<PublicKey>()
            .unwrap();
        let authority = Authority::new(3).add_key(key, 2).add_account("bob", 1);
        let keys = HashSet::from([key]);
        assert!(!authority.is_satisfied(&keys));
        assert!(authority.is_satisfied_with_accounts(&keys, &|name| {
            if name == "bob" {
                Some(Authority::new(1).add_key(key, 1))
            } else {
                None
            }
        }));
    }
}
