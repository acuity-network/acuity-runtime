set shell := ["nu", "-c"]

runtime_wasm := "target/release/wbuild/acuity-runtime/acuity_runtime.wasm"
chain_spec := "target/dev-chain-spec.json"

default:
    @just _task-selector

_task-selector:
    #!/usr/bin/env nu
    let selected_task = (
        just --summary -u
        | split row ' '
        | to text
        | fzf --header 'Available recipes' --header-first --layout reverse --preview 'just --show {}'
        | if ($in | is-empty) { 'about' } else { $in }
    )
    just $selected_task

menu:
    @just _task-selector

@about:
    just --list

build:
    cargo build --release

build-benchmarks:
    cargo build --release --features runtime-benchmarks

chain-spec: build
    if ((which polkadot-omni-node | length) == 0) { error make { msg: 'polkadot-omni-node is not installed or not on PATH' } }
    polkadot-omni-node chain-spec-builder \
      --chain-spec-path {{chain_spec}} \
      create \
      -t development \
      --relay-chain rococo-local \
      --runtime {{runtime_wasm}} \
      named-preset development
    print $'Wrote chain spec to {{chain_spec}}'

dev-node: chain-spec
    if ((which polkadot-omni-node | length) == 0) { error make { msg: 'polkadot-omni-node is not installed or not on PATH' } }
    polkadot-omni-node \
      --chain {{chain_spec}} \
      --dev \
      --dev-block-time 1000 \
      --blocks-pruning archive-canonical \
      --state-pruning archive-canonical

benchmark:
    if ((which frame-omni-bencher | length) == 0) { error make { msg: 'frame-omni-bencher is not installed or not on PATH' } }
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
