#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RUNTIME_WASM="$REPO_ROOT/target/release/wbuild/acuity-runtime/acuity_runtime.wasm"
CHAIN_SPEC="$REPO_ROOT/target/dev-chain-spec.json"

if ! command -v cargo >/dev/null 2>&1; then
  printf 'error: cargo is not installed or not on PATH\n' >&2
  exit 1
fi

if ! command -v polkadot-omni-node >/dev/null 2>&1; then
  printf 'error: polkadot-omni-node is not installed or not on PATH\n' >&2
  exit 1
fi

cd "$REPO_ROOT"

cargo build --release

polkadot-omni-node chain-spec-builder \
  --chain-spec-path "$CHAIN_SPEC" \
  create \
  --relay-chain rococo-local \
  --runtime "$RUNTIME_WASM" \
  named-preset development

exec polkadot-omni-node \
  --chain "$CHAIN_SPEC" \
  --dev \
  --dev-block-time 1000 \
  --blocks-pruning archive-canonical
