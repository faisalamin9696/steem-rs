# Changelog

All notable changes to this workspace are documented here.

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
