# Acuity Runtime Architecture

## Overview

Acuity Runtime is a **Substrate/Polkadot parachain runtime** written in Rust. It compiles to a Wasm blob for on-chain execution and a native `rlib` for tooling. The runtime targets `polkadot-stable2603` and uses the FRAME framework from the Polkadot SDK.

## Tech Stack

| Component | Technology |
|---|---|
| Language | Rust (edition 2024) |
| Framework | Substrate/Polkadot SDK (FRAME) |
| Encoding | Parity Scale Codec v3.7.5, scale-info v2.11.6 |
| Serialization | serde_json v1.0.145 (genesis presets) |
| Build | Cargo + `substrate-wasm-builder` |
| Task Runner | `just` (shell: Nushell) |
| Node Runner | `polkadot-omni-node` |
| Benchmarking | `frame-omni-bencher` |
| Output | `cdylib` (Wasm blob) + `rlib` (native lib) |

## Directory Structure

```
acuity-runtime/
├── src/
│   ├── lib.rs              # Runtime definition, pallet configs, APIs, genesis, tests
│   └── weights/             # Benchmark-derived weight files per pallet
│       ├── mod.rs
│       ├── frame_system.rs
│       ├── pallet_balances.rs
│       ├── pallet_content.rs
│       ├── pallet_account_content.rs
│       ├── pallet_account_profile.rs
│       ├── pallet_content_reactions.rs
│       └── generated/       # Placeholder for generated weight output
├── templates/
│   └── runtime-weight-template.hbs  # Handlebars template for weight file generation
├── scripts/                  # Placeholder for future scripts
├── build.rs                  # Build script: patches frame-benchmarking + invokes wasm-builder
├── Cargo.toml                # Crate manifest
├── justfile                  # Task recipes (build, chain-spec, dev-node, benchmark)
└── README.md
```

## Pallet Architecture

The runtime wires together 12 pallets in `src/lib.rs` via the `#[frame_construct_runtime]` macro:

| Index | Pallet | Source | Responsibility |
|---|---|---|---|
| 0 | `System` | polkadot-sdk | Block execution, events, account nonce |
| 1 | `Timestamp` | polkadot-sdk | On-chain timestamp; drives Aura slot duration |
| 2 | `ParachainSystem` | polkadot-sdk (cumulus) | Parachain identity and consensus data |
| 3 | `Aura` | polkadot-sdk | Block authorship consensus (sr25519 authorities) |
| 4 | `Balances` | polkadot-sdk | Token balances, transfers, existential deposit |
| 5 | `Sudo` | polkadot-sdk | Superuser key (dev mode) |
| 6 | `TransactionPayment` | polkadot-sdk | Transaction fee computation |
| 7 | `Content` | acuity-network/pallet-content | Publish, revise, retract content items (IPFS hashes) |
| 8 | `AccountContent` | acuity-network/pallet-content | Maps accounts to owned content items |
| 9 | `AccountProfile` | acuity-network/pallet-content | Sets an account's profile to a content item |
| 10 | `ContentReactions` | acuity-network/pallet-content | Emoji reactions on content items (per revision) |
| 11 | `Utility` | polkadot-sdk | Batch/union dispatch of calls |

### Cross-Pallet Dependencies

```
Content ──────────────────┐
  │ ItemState storage       │
  │                         ▼
  │              AccountContent (verify ownership)
  │              AccountProfile (verify ownership)
  │              ContentReactions (verify item exists)
  │
Balances ←── System (AccountStore)
Timestamp ──→ Aura (OnTimestampSet)
Balances ←── TransactionPayment (FungibleAdapter for fees)
```

- `AccountContent`, `AccountProfile`, and `ContentReactions` all read `Content::ItemState` storage to verify item existence and ownership before mutating their own storage.
- `Balances` stores `AccountData` inside `System` via `type AccountStore = System`.
- `Timestamp` drives `Aura` via `type OnTimestampSet = Aura`.

## Runtime Configuration

### Constants

| Constant | Value | Used By |
|---|---|---|
| `MILLI_SECS_PER_BLOCK` | 6000 (6s) | Timestamp/Aura slot duration |
| `ItemIdNamespace` | 1000 | Content item ID derivation |
| `MaxParents` | 32 | Content item parent links |
| `MaxLinks` | 128 | Content item links |
| `MaxMentions` | 256 | Content item mentions |
| `MaxItemsPerAccount` | 1024 | Account content item limit |
| `MaxEmojis` | 16 | Reaction emoji limit per user per (item, revision) |
| `ExistentialDeposit` | 1 | Balances minimum |

### Fee Model (dev)

- **Weight-to-fee**: `NoFee` (zero) — no cost for computation weight
- **Length-to-fee**: `ConstantMultiplier<Balance, ConstU128<1>>` — 1 unit per byte

### Genesis Preset

The `development` preset configures:
- Alice and Bob with `1_000_000_000_000_000` balance each
- Alice as sole Aura authority
- Alice as Sudo key
- `ParaId(1000)` for parachain identity

## Transaction Pipeline

Extrinsics pass through `TxExtension` (signed transaction extension pipeline):

```
AuthorizeCall → CheckNonZeroSender → CheckSpecVersion → CheckTxVersion
→ CheckGenesis → CheckEra → CheckNonce → CheckWeight → ChargeTransactionPayment
→ WeightReclaim
```

## Runtime APIs

Implemented in `impl_runtime_apis!`:

- **Core** — version, execute_block, initialize_block
- **Metadata** — metadata, metadata_at_version, metadata_versions
- **BlockBuilder** — apply_extrinsic, finalize_block, inherent_extrinsics, check_inherents
- **TaggedTransactionQueue** — validate_transaction
- **OffchainWorkerApi** — offchain_worker
- **SessionKeys** — generate_session_keys, decode_session_keys
- **AuraApi** — slot_duration, authorities
- **GetParachainInfo** — parachain_id
- **AccountNonceApi** — account_nonce
- **TransactionPaymentApi** — query_info, query_fee_details, query_weight_to_fee, query_length_to_fee
- **GenesisBuilder** — build_state, get_preset, preset_names
- **Benchmark** (feature-gated) — benchmark_metadata, dispatch_benchmark

## Data Flows

### Content Publication

1. `Content::publish_item(origin, nonce, parents, flags, links, mentions, ipfs_hash)`
2. Item ID derived via `Blake2-256(account, nonce, ItemIdNamespace)`
3. `Content::ItemState` populated with owner, flags, revision_id
4. `Content::publish_revision` bumps revision_id
5. `Content::retract_item` sets `RETRACTED` flag (blocks further revisions)
6. `Content::set_not_revisionable` / `set_not_retractable` remove flags

### Account Content & Profile

1. `AccountContent::add_item(origin, item_id)` — verifies ownership via `Content::ItemState`, adds to account list
2. `AccountProfile::set_profile(origin, item_id)` — verifies ownership, sets as profile
3. Only the item owner can add or profile their items (`WrongAccount` error otherwise)
4. `AccountContent::remove_item` removes item from account list

### Content Reactions

1. `ContentReactions::add_reaction(origin, item_id, revision_id, Emoji)` — verifies item exists
2. Reactions are per-revision: different revisions can have different reactions from the same user
3. Maximum `MaxEmojis` (16) emojis per account per (item, revision)
4. `ContentReactions::remove_reaction` removes a specific emoji

## Build System

### Build Process

1. `cargo build --release` compiles the runtime
2. `build.rs` runs first: patches `frame-benchmarking` no-std bug, then invokes `substrate-wasm-builder`
3. Output: `target/release/wbuild/acuity-runtime/acuity_runtime.wasm`

### Build Patch

`build.rs` patches `frame-benchmarking`'s `current_time()` function to return `0u128.to_le_bytes()` in `no_std` mode, allowing Wasm benchmark builds to succeed. This workaround can be removed once the upstream issue is fixed.

### Task Recipes (`justfile`)

| Recipe | Purpose |
|---|---|
| `build` | Compile runtime (`cargo build --release`) |
| `build-benchmarks` | Compile with `runtime-benchmarks` feature |
| `chain-spec` | Generate dev chain spec using built Wasm |
| `dev-node` | Start local dev node (1s blocks, archive pruning) |
| `benchmark` | Full benchmark cycle: build, run bencher, generate weights, rebuild |

### Benchmarking Pipeline

1. `just build-benchmarks` enables `runtime-benchmarks` feature
2. `frame-omni-bencher v1 benchmark pallet` runs against the Wasm blob using `templates/runtime-weight-template.hbs`
3. Template generates Rust weight files implementing each pallet's `WeightInfo` trait
4. The template has special-case handling for `publish_item` and `publish_revision` parameter names
4. `just build` recompiles with the generated weights

## `no_std` Architecture

- `#![cfg_attr(not(feature = "std"), no_std)]` — compiles to Wasm without `std`
- Uses `extern crate alloc` and `alloc::vec::Vec`
- Wasm binary included via `include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"))` behind `#[cfg(feature = "std")]`

## Testing

Integration tests in `src/lib.rs` cover:

1. **Genesis presets** — development config contains expected values
2. **Runtime version** — spec/impl names and versions are consistent
3. **Configuration constants** — all `parameter_types!` match expected values
4. **Transaction fees** — zero weight fee, linear length fee
5. **Content flow** — publish, revise, retract, and retraction blocking
6. **Account content/profile** — ownership verification, add/remove items, set profile
7. **Content reactions** — per-revision tracking, emoji limits, removal
8. **Content bounds** — BoundedVec limits for parents, links, mentions