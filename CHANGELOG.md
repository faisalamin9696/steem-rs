# Changelog

All notable changes to this workspace are documented here.

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
