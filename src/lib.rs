#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

extern crate alloc;

use alloc::vec::Vec;
use pallet_transaction_payment::{FeeDetails, RuntimeDispatchInfo};
use polkadot_sdk::{
    polkadot_sdk_frame::{
        self as frame,
        deps::sp_genesis_builder,
        runtime::{apis, prelude::*},
    },
    staging_parachain_info as parachain_info, *,
};

pub mod genesis_config_presets {
    use super::*;
    use crate::{BalancesConfig, ParachainSystemConfig, RuntimeGenesisConfig, SudoConfig};
    use alloc::vec;
    use cumulus_primitives_core::ParaId;
    use serde_json::Value;
    use sp_consensus_aura::sr25519::AuthorityId as AuraId;
    use sp_keyring::Sr25519Keyring;

    fn dev_authority() -> AuraId {
        Sr25519Keyring::Alice.public().into()
    }

    pub fn development_config_genesis() -> Value {
        frame_support::build_struct_json_patch!(RuntimeGenesisConfig {
            balances: BalancesConfig {
                balances: vec![
                    (Sr25519Keyring::Alice.to_account_id(), 1_000_000_000_000_000),
                    (Sr25519Keyring::Bob.to_account_id(), 1_000_000_000_000_000),
                ]
            },
            aura: pallet_aura::GenesisConfig {
                authorities: vec![dev_authority()],
            },
            parachain_system: ParachainSystemConfig {
                parachain_id: ParaId::new(1000),
            },
            sudo: SudoConfig {
                key: Some(Sr25519Keyring::Alice.to_account_id()),
            },
        })
    }

    pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
        let patch = match id.as_ref() {
            sp_genesis_builder::DEV_RUNTIME_PRESET => development_config_genesis(),
            _ => return None,
        };

        Some(serde_json::to_string(&patch).ok()?.into_bytes())
    }

    pub fn preset_names() -> Vec<PresetId> {
        alloc::vec![PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET)]
    }
}

pub mod weights;

#[runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: alloc::borrow::Cow::Borrowed("acuity-runtime"),
    impl_name: alloc::borrow::Cow::Borrowed("acuity-runtime"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    system_version: 1,
};

#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

type TxExtension = (
    frame_system::AuthorizeCall<Runtime>,
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
    frame_system::WeightReclaim<Runtime>,
);

const MILLI_SECS_PER_BLOCK: u64 = 6000;

#[frame_construct_runtime]
mod runtime {
    #[runtime::runtime]
    #[runtime::derive(
        RuntimeCall,
        RuntimeEvent,
        RuntimeError,
        RuntimeOrigin,
        RuntimeFreezeReason,
        RuntimeHoldReason,
        RuntimeSlashReason,
        RuntimeLockId,
        RuntimeTask,
        RuntimeViewFunction
    )]
    pub struct Runtime;

    #[runtime::pallet_index(0)]
    pub type System = frame_system::Pallet<Runtime>;

    #[runtime::pallet_index(1)]
    pub type Timestamp = pallet_timestamp::Pallet<Runtime>;

    #[runtime::pallet_index(2)]
    pub type ParachainSystem = parachain_info::Pallet<Runtime>;

    #[runtime::pallet_index(3)]
    pub type Aura = pallet_aura::Pallet<Runtime>;

    #[runtime::pallet_index(4)]
    pub type Balances = pallet_balances::Pallet<Runtime>;

    #[runtime::pallet_index(5)]
    pub type Sudo = pallet_sudo::Pallet<Runtime>;

    #[runtime::pallet_index(6)]
    pub type TransactionPayment = pallet_transaction_payment::Pallet<Runtime>;

    #[runtime::pallet_index(7)]
    pub type Content = pallet_content::Pallet<Runtime>;
}

parameter_types! {
    pub const Version: RuntimeVersion = VERSION;
    pub const MaxParents: u32 = 32;
    pub const MaxLinks: u32 = 128;
}

#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
    type Block = Block;
    type Version = Version;
    type AccountData = pallet_balances::AccountData<<Runtime as pallet_balances::Config>::Balance>;
    type SystemWeightInfo = weights::frame_system::WeightInfo<Runtime>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Runtime {
    type AccountStore = System;
    type WeightInfo = weights::pallet_balances::WeightInfo<Runtime>;
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = sp_consensus_aura::sr25519::AuthorityId;
    type DisabledValidators = ();
    type MaxAuthorities = ConstU32<32>;
    type AllowMultipleBlocksPerSlot = ConstBool<false>;
    type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
}

impl parachain_info::Config for Runtime {}

#[derive_impl(pallet_sudo::config_preludes::TestDefaultConfig)]
impl pallet_sudo::Config for Runtime {}

impl pallet_timestamp::Config for Runtime {
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = ConstU64<{ MILLI_SECS_PER_BLOCK / 2 }>;
    type WeightInfo = ();
}

#[derive_impl(pallet_transaction_payment::config_preludes::TestDefaultConfig)]
impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, ()>;
    type WeightToFee = NoFee<<Self as pallet_balances::Config>::Balance>;
    type LengthToFee = FixedFee<1, <Self as pallet_balances::Config>::Balance>;
}

impl pallet_content::Config for Runtime {
    type WeightInfo = weights::pallet_content::WeightInfo<Runtime>;
    type MaxParents = MaxParents;
    type MaxLinks = MaxLinks;
}

#[cfg(feature = "runtime-benchmarks")]
impl frame_system_benchmarking::Config for Runtime {}

type Block = frame::runtime::types_common::BlockOf<Runtime, TxExtension>;
type Header = HeaderFor<Runtime>;
type RuntimeExecutive =
    Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPalletsWithSystem>;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    use super::*;
    use frame_support::traits::{StorageInfo, WhitelistedStorageKeys};
    use polkadot_sdk::frame_system_benchmarking::Pallet as SystemBench;

    polkadot_sdk::frame_benchmarking::define_benchmarks!(
        [frame_system, SystemBench::<Runtime>]
        [pallet_balances, Balances]
        [pallet_content, Content]
    );

    pub fn benchmark_metadata(
        extra: bool,
    ) -> (Vec<frame_benchmarking::BenchmarkList>, Vec<StorageInfo>) {
        let mut list = Vec::<frame_benchmarking::BenchmarkList>::new();
        list_benchmarks!(list, extra);

        let storage_info = AllPalletsWithSystem::storage_info();
        (list, storage_info)
    }

    pub fn dispatch_benchmark(
        config: frame_benchmarking::BenchmarkConfig,
    ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, alloc::string::String> {
        let whitelist = AllPalletsWithSystem::whitelisted_storage_keys();
        let mut batches = Vec::<frame_benchmarking::BenchmarkBatch>::new();
        let params = (&&config, &whitelist);
        add_benchmarks!(params, batches);
        Ok(batches)
    }
}

impl_runtime_apis! {
    impl apis::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: <Block as frame::traits::Block>::LazyBlock) {
            RuntimeExecutive::execute_block(block)
        }

        fn initialize_block(header: &Header) -> ExtrinsicInclusionMode {
            RuntimeExecutive::initialize_block(header)
        }
    }

    impl apis::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }

        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }

        fn metadata_versions() -> Vec<u32> {
            Runtime::metadata_versions()
        }
    }

    impl apis::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: ExtrinsicFor<Runtime>) -> ApplyExtrinsicResult {
            RuntimeExecutive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> HeaderFor<Runtime> {
            RuntimeExecutive::finalize_block()
        }

        fn inherent_extrinsics(data: InherentData) -> Vec<ExtrinsicFor<Runtime>> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: <Block as frame::traits::Block>::LazyBlock,
            data: InherentData,
        ) -> CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl apis::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: ExtrinsicFor<Runtime>,
            block_hash: <Runtime as frame_system::Config>::Hash,
        ) -> TransactionValidity {
            RuntimeExecutive::validate_transaction(source, tx, block_hash)
        }
    }

    impl apis::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &HeaderFor<Runtime>) {
            RuntimeExecutive::offchain_worker(header)
        }
    }

    impl apis::SessionKeys<Block> for Runtime {
        fn generate_session_keys(_: Option<Vec<u8>>) -> Vec<u8> {
            Default::default()
        }

        fn decode_session_keys(_: Vec<u8>) -> Option<Vec<(Vec<u8>, apis::KeyTypeId)>> {
            Default::default()
        }
    }

    impl sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }

        fn authorities() -> Vec<sp_consensus_aura::sr25519::AuthorityId> {
            pallet_aura::Authorities::<Runtime>::get().into_inner()
        }
    }

    impl cumulus_primitives_core::GetParachainInfo<Block> for Runtime {
        fn parachain_id() -> cumulus_primitives_core::ParaId {
            ParachainSystem::parachain_id()
        }
    }

    impl apis::AccountNonceApi<Block, interface::AccountId, interface::Nonce> for Runtime {
        fn account_nonce(account: interface::AccountId) -> interface::Nonce {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, interface::Balance>
        for Runtime {
        fn query_info(
            uxt: ExtrinsicFor<Runtime>,
            len: u32,
        ) -> RuntimeDispatchInfo<interface::Balance> {
            TransactionPayment::query_info(uxt, len)
        }

        fn query_fee_details(
            uxt: ExtrinsicFor<Runtime>,
            len: u32,
        ) -> FeeDetails<interface::Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }

        fn query_weight_to_fee(weight: Weight) -> interface::Balance {
            TransactionPayment::weight_to_fee(weight)
        }

        fn query_length_to_fee(length: u32) -> interface::Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    impl apis::GenesisBuilder<Block> for Runtime {
        fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
            build_state::<RuntimeGenesisConfig>(config)
        }

        fn get_preset(id: &Option<PresetId>) -> Option<Vec<u8>> {
            get_preset::<RuntimeGenesisConfig>(id, self::genesis_config_presets::get_preset)
        }

        fn preset_names() -> Vec<PresetId> {
            self::genesis_config_presets::preset_names()
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(
            extra: bool,
        ) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            benches::benchmark_metadata(extra)
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig,
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, alloc::string::String> {
            benches::dispatch_benchmark(config)
        }
    }
}

pub mod interface {
    use super::Runtime;
    use polkadot_sdk::{polkadot_sdk_frame as frame, *};

    pub type Block = super::Block;
    pub use frame::runtime::types_common::OpaqueBlock;
    pub type AccountId = <Runtime as frame_system::Config>::AccountId;
    pub type Nonce = <Runtime as frame_system::Config>::Nonce;
    pub type Hash = <Runtime as frame_system::Config>::Hash;
    pub type Balance = <Runtime as pallet_balances::Config>::Balance;
}
