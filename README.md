# Acuity Runtime

This repo contains a minimal FRAME runtime called `Acuity Runtime` (`acuity-runtime`), intended to be run with `polkadot-omni-node`.

## Build the runtime

```bash
cargo build --release
```

This produces the wasm blob at:

`target/release/wbuild/acuity-runtime/acuity_runtime.wasm`

## Generate a chain spec

```bash
  polkadot-omni-node chain-spec-builder create \
  --relay-chain rococo-local \
  --runtime target/release/wbuild/acuity-runtime/acuity_runtime.wasm \
  named-preset development > chain_spec.json
```

## Run with Omni Node

```bash
polkadot-omni-node --chain chain_spec.json --dev --dev-block-time 1000
```

The node will run in dev/manual-seal style mode and produce blocks locally.

## Benchmark with frame-omni-bencher

### Current status in this repository

`frame-omni-bencher` is installed.

This repository now includes runtime benchmarking wiring (`runtime-benchmarks` feature plus the `frame_benchmarking::Benchmark` runtime API implementation).

With `polkadot-stable2512-3`, there is an upstream `frame-benchmarking` no-std issue in Wasm benchmark builds.

This repository applies a temporary source patch automatically in `build.rs` before calling `substrate-wasm-builder`, so no manual machine-local patching is required.

### Expected benchmarking workflow

Use this flow:

1. Build a benchmark-enabled Wasm runtime.
2. List available benchmarks.
3. Run the benchmark script to benchmark all pallets and write runtime-ready weight code.

Example commands:

```bash
# 1) Build benchmark-enabled runtime
cargo build --release --features runtime-benchmarks

# 2) List available pallet/extrinsic benchmarks
frame-omni-bencher v1 benchmark pallet \
  --runtime target/release/wbuild/acuity-runtime/acuity_runtime.wasm \
  --list=all

# 3) Benchmark all runtime pallets and write weight files into src/weights
./scripts/benchmark-runtime.sh

# 4) Verify runtime still builds with generated weights
cargo build --release
```

### Notes

- `--steps` controls how many points are sampled across benchmark ranges.
- `--repeat` controls how many in-Wasm repetitions are run per sampled point.
- The script benchmarks `frame_system`, `pallet_balances`, and `pallet_content` together from this runtime Wasm.
- The script uses `templates/runtime-weight-template.hbs`, which is runtime-specific and emits code that compiles in this runtime without post-processing.
- Generated files are written directly to `src/weights/frame_system.rs`, `src/weights/pallet_balances.rs`, and `src/weights/pallet_content.rs`.
- If upstream fixes `frame-benchmarking` on your chosen `polkadot-sdk` tag, you can remove the temporary auto-patch in `build.rs`.
