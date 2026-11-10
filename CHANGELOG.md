# Changelog

All notable changes to this workspace are documented here.

## 0.1.2 - 2026-11-10

### Added

- `Operation::from_tuple` JSON parser for wallet operations.
- `set_withdraw_vesting_route`, `witness_update`,
  `update_proposal_votes`, `change_recovery_account`,
  `request_account_recovery`, `cancel_transfer_from_savings` and
  `account_witness_proxy`.
- `Transaction::from_chain`, `Transaction::from_json_value`,
  `expiration_from_chain_time` and `transaction_id`.
- Ignored live-node smoke tests for the RPC client and live transaction
  build/sign path.

## 0.1.1 - 2026-11-10

### Fixed

- Corrected the `repository`/`homepage` URLs to
  `https://github.com/faisalamin9696/steem-rs`.
- Declared the real MSRV: Rust 1.88 (required by `icu_*` 2.4).
- Allowed `CDLA-Permissive-2.0` in `cargo-deny` for `webpki-roots`.

## 0.1.0 - 2026-11-10

### Added

- `steem-core`: account names, assets, authorities, Graphene wire codec,
  WIF/STM keys, transaction digest/signing, canonical signatures and
  Steem-compatible memo encryption.
- `steem-rpc`: blocking JSON-RPC client with typed errors, dynamic global
  properties, accounts, version/chain-id and broadcast methods.
- `steem`: umbrella crate re-exporting the core and RPC crates.
- Byte-for-byte compatibility tests against dsteem for transaction bytes,
  transaction digests and memo ciphertexts.
