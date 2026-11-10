# steem-core

Core Steem protocol primitives for Rust:

- `AccountName`, `Asset` and `Authority` types
- Graphene binary codec (varint, vstring, little-endian)
- WIF private keys and `STM` public keys
- transaction digest, signing, canonical signatures and recovery
- Steem-compatible memo encryption (ECDH + SHA-512 + AES-256-CBC)
- authority weight/threshold evaluation

Part of the [steem-rs](https://github.com/faisalamin9696/steem-rs) workspace.

## Example

```rust
use steem_core::{Asset, ChainId, Operation, PrivateKey, Transaction};

let key = PrivateKey::from_wif("5K...")?;
let tx = Transaction::new(
    1234,
    567_890_123,
    "2026-11-10T12:00:00",
    vec![Operation::Transfer {
        from: "alice".into(),
        to: "bob".into(),
        amount: "1.000 STEEM".parse::<Asset>()?,
        memo: "hello".into(),
    }],
);
let signed = tx.sign(&key, &ChainId::MAINNET)?;
```

## Security

`PrivateKey` zeroizes its secret on drop and does not implement `Debug` or
`Serialize`. Signing is local; the RPC transport lives in `steem-rpc`.

## License

MIT OR Apache-2.0.
