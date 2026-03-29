set shell := ["bash", "-euo", "pipefail", "-c"]

runtime_wasm := "target/release/wbuild/acuity-runtime/acuity_runtime.wasm"
chain_spec := "target/dev-chain-spec.json"

default:
    @just --list

build:
    cargo build --release

build-benchmarks:
    cargo build --release --features runtime-benchmarks

chain-spec: build
    if ! command -v polkadot-omni-node >/dev/null 2>&1; then printf 'error: polkadot-omni-node is not installed or not on PATH\n' >&2; exit 1; fi
    polkadot-omni-node chain-spec-builder \
      --chain-spec-path {{chain_spec}} \
      create \
      --relay-chain rococo-local \
      --runtime {{runtime_wasm}} \
      named-preset development

dev-node: chain-spec
    if ! command -v polkadot-omni-node >/dev/null 2>&1; then printf 'error: polkadot-omni-node is not installed or not on PATH\n' >&2; exit 1; fi
    polkadot-omni-node \
      --chain {{chain_spec}} \
      --dev \
      --dev-block-time 1000 \
      --blocks-pruning archive-canonical

benchmark:
    if ! command -v frame-omni-bencher >/dev/null 2>&1; then printf 'error: frame-omni-bencher is not installed or not on PATH\n' >&2; exit 1; fi
    cargo build --release --features runtime-benchmarks
    frame-omni-bencher v1 benchmark pallet \
      --runtime {{runtime_wasm}} \
      --all \
      --steps 50 \
      --repeat 20 \
      --template templates/runtime-weight-template.hbs \
      --output src/weights \
      --quiet
    cargo build --release

run: dev-node

bench: benchmark
