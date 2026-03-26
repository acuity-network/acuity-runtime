#!/usr/bin/env bash
set -euo pipefail

RUNTIME_WASM="target/release/wbuild/acuity-runtime/acuity_runtime.wasm"

cargo build --release --features runtime-benchmarks

frame-omni-bencher v1 benchmark pallet \
  --runtime "$RUNTIME_WASM" \
  --all \
  --steps 50 \
  --repeat 20 \
  --template templates/runtime-weight-template.hbs \
  --output src/weights \
  --quiet

cargo build --release
