# Simple Omni Runtime

This repo contains a minimal FRAME runtime intended to be run with `polkadot-omni-node`.

## Build the runtime

```bash
cargo build --release
```

This produces the wasm blob at:

`target/release/wbuild/simple-omni-runtime/simple_omni_runtime.wasm`

## Generate a chain spec

```bash
polkadot-omni-node chain-spec-builder create \
  --relay-chain rococo-local \
  --runtime target/release/wbuild/simple-omni-runtime/simple_omni_runtime.wasm \
  named-preset development > chain_spec.json
```

## Run with Omni Node

```bash
polkadot-omni-node --chain chain_spec.json --dev --dev-block-time 1000
```

The node will run in dev/manual-seal style mode and produce blocks locally.
