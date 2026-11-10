# steem-rpc

A small blocking JSON-RPC client for Steem nodes.

It never sees private keys: it only accepts already-signed transactions from
`steem-core`.

```rust
use steem_rpc::RpcClient;

let client = RpcClient::new("https://api.steemit.com")?;
let props = client.get_dynamic_global_properties()?;
println!("head block: {}", props.head_block_number);
```

Part of the [steem-rs](https://github.com/faisalamin9696/steem-rs) workspace.

## License

MIT OR Apache-2.0.
