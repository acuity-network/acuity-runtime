# Acuity Runtime

This repo contains a minimal FRAME runtime called `Acuity Runtime` (`acuity-runtime`), intended to be run with `polkadot-omni-node`.

## Build the runtime

```bash
just build
```

This produces the wasm blob at:

`target/release/wbuild/acuity-runtime/acuity_runtime.wasm`

## Generate a chain spec

```bash
just chain-spec
```

## Run with Omni Node

Use `just` to rebuild the runtime, regenerate the chain spec, and start a local dev node:

```bash
just dev-node
```

The recipe writes a fresh chain spec to `target/dev-chain-spec.json` and starts `polkadot-omni-node` with `--dev`, `--dev-block-time 1000`, and `--blocks-pruning archive-canonical`.

If you want to see the available commands, use:

```bash
just --list
```

The underlying manual steps are:

```bash
just build

polkadot-omni-node chain-spec-builder \
  --chain-spec-path target/dev-chain-spec.json \
  create \
  -t development \
  --relay-chain rococo-local \
  --runtime target/release/wbuild/acuity-runtime/acuity_runtime.wasm \
  named-preset development

polkadot-omni-node \
  --chain target/dev-chain-spec.json \
  --dev \
  --dev-block-time 1000 \
  --blocks-pruning archive-canonical
```

The node will run in dev/manual-seal style mode and produce blocks locally.
The chain-spec command prints the output path when generation completes.

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
3. Run the benchmark recipe to benchmark all pallets and write runtime-ready weight code.

Example commands:

```bash
# 1) Build benchmark-enabled runtime
just build-benchmarks

# 2) List available pallet/extrinsic benchmarks
frame-omni-bencher v1 benchmark pallet \
  --runtime target/release/wbuild/acuity-runtime/acuity_runtime.wasm \
  --list=all

# 3) Benchmark all runtime pallets and write weight files into src/weights
just benchmark

# 4) Verify runtime still builds with generated weights
just build
```

### Notes

- `--steps` controls how many points are sampled across benchmark ranges.
- `--repeat` controls how many in-Wasm repetitions are run per sampled point.
- The benchmark recipe runs `frame-omni-bencher` against the built runtime Wasm and writes generated files into `src/weights`.
- The script uses `templates/runtime-weight-template.hbs`, which is runtime-specific and emits code that compiles in this runtime without post-processing.
- Generated files are written directly to `src/weights/frame_system.rs`, `src/weights/pallet_balances.rs`, and `src/weights/pallet_content.rs`.
- If upstream fixes `frame-benchmarking` on your chosen `polkadot-sdk` tag, you can remove the temporary auto-patch in `build.rs`.
