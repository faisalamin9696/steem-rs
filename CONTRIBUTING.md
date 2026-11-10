# Contributing

## Development

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Guidelines

- Keep `steem-core` transport-free. RPC belongs in `steem-rpc`.
- Never add `Debug`, `Serialize` or `Clone` to private-key material without a
  security review. `PrivateKey` is intentionally non-`Debug`/non-`Serialize`.
- Add a test vector for every wire-format change. The existing vectors come
  from dsteem and are checked byte-for-byte.
- Prefer explicit errors over panics on untrusted input.
- Run `cargo audit` and `cargo deny check` before proposing a release.

## Commits and releases

Use focused commits with a clear rationale. Releases are prepared with:

```sh
cargo package --list -p steem-core
cargo publish --dry-run -p steem-core
```

Publish in dependency order: `steem-core`, then `steem-rpc`, then `steem`.
