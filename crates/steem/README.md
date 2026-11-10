# steem

Umbrella crate for the Rust Steem SDK. It re-exports `steem-core` and, with the
default `rpc` feature, `steem-rpc`.

```rust
use steem::{Asset, ChainId, Operation, PrivateKey, Transaction};

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

Part of the [steem-rs](https://github.com/faisalamin9696/steem-rs) workspace.

## License

MIT OR Apache-2.0.
