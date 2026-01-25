# GAME TITLE

TODO: link

## Running from source

### Natively

```bash
# run in release
$ cargo run --release
```

TODO: document any optional feature flags

### WASM

Install prerequisites:
```bash
$ rustup target add wasm32-unknown-unknown
$ cargo install wasm-bindgen-cli
$ cargo install basic-http-server
```

```bash
$ RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --profile wasm-release --target wasm32-unknown-unknown
$ wasm-bindgen --out-name jam7 \
  --out-dir wasm/target \
  --target web target/wasm32-unknown-unknown/wasm-release/bevy-jam-7.wasm

$ basic-http-server wasm
```


## Known issues:
TODO


## Credits:
TODO

