//! Configuration of the pallets used in the runtime.
//! The pallets used in the runtime are configured here.
//! This file is used to generate the `construct_runtime!` macro.
#[cfg(all(debug_assertions, feature = "dev"))]
use std::env::VarError;
use std::ops::Deref;
#[cfg(all(debug_assertions, feature = "dev"))]
use std::path::Path;
use std::sync::Arc;

use blockifier::versioned_constants::VersionedConstants;
pub use frame_support::traits::{
    ConstBool, ConstU128, ConstU32, ConstU64, ConstU8, KeyOwnerProofSystem, OnTimestampSet, Randomness, StorageInfo,
};
pub use frame_support::weights::constants::{
    BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
};
pub use frame_support::weights::{IdentityFee, Weight};
pub use frame_support::{construct_runtime, parameter_types, StorageValue};
pub use frame_system::Call as SystemCall;
use lazy_static::lazy_static;
pub use mp_chain_id::SN_GOERLI_CHAIN_ID;
pub use mp_program_hash::SN_OS_PROGRAM_HASH;
/// Import the StarkNet pallet.
pub use pallet_starknet;
pub use pallet_timestamp::Call as TimestampCall;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_runtime::traits::{AccountIdLookup, BlakeTwo256};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};
use sp_std::marker::PhantomData;
use frame_system::{
    self as system,
};
pub use pallet_nostr;
use crate::*;
use sp_runtime::generic::Era;
use sp_runtime::traits;
pub use pallet_balances;
use pallet_balances::AccountData;
use parity_scale_codec::{ Encode};

// Configure FRAME pallets to include in runtime.

// --------------------------------------
// CUSTOM PALLETS
// --------------------------------------
const EXECUTION_CONSTANTS_STR: &str = include_str!("../resources/versioned_constants.json");

#[cfg(not(all(debug_assertions, feature = "dev")))]
lazy_static! {
    static ref EXECUTION_CONSTANTS: Arc<VersionedConstants> = serde_json::from_str(EXECUTION_CONSTANTS_STR).unwrap();
}

#[cfg(all(debug_assertions, feature = "dev"))]
lazy_static! {
    static ref EXECUTION_CONSTANTS: Arc<VersionedConstants> = Arc::new(
        std::env::var("EXECUTION_CONSTANTS_PATH")
            .map(|path| {
                VersionedConstants::try_from(Path::new(path.as_str()))
                    .expect("Failed to load execution constants from path")
            })
            .unwrap_or_else(|e| {
                match e {
                    VarError::NotPresent => serde_json::from_str(EXECUTION_CONSTANTS_STR).unwrap(),
                    VarError::NotUnicode(_) => panic!("Failed to load execution constants variable"),
                }
            })
    );
}

/// Configure the Starknet pallet in pallets/starknet.
impl pallet_starknet::Config for Runtime {
    type TimestampProvider = Timestamp;
    type UnsignedPriority = UnsignedPriority;
    type TransactionLongevity = TransactionLongevity;
    #[cfg(not(feature = "disable-transaction-fee"))]
    type DisableTransactionFee = ConstBool<false>;
    #[cfg(feature = "disable-transaction-fee")]
    type DisableTransactionFee = ConstBool<true>;
    type DisableNonceValidation = ConstBool<false>;
    type ProtocolVersion = ProtocolVersion;
    type ProgramHash = ProgramHash;
    type ExecutionConstants = ExecutionConstants;
    type InvokeTransactionFilter = ();
    type DeclareTransactionFilter = ();
    type DeployAccountTransactionFilter = ();
}

/// --------------------------------------
/// FRAME SYSTEM PALLET
/// --------------------------------------

/// Configuration of `frame_system` pallet.
impl frame_system::Config for Runtime {
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = frame_support::traits::Everything;
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = BlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = BlockLength;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type RuntimeCall = RuntimeCall;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = AccountIdLookup<AccountId, ()>;
    /// The index type for storing how many extrinsics an account has signed.
    type Nonce = Index;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The Block type.
    type Block = Block;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    /// The ubiquitous origin type.
    type RuntimeOrigin = RuntimeOrigin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// Version of the runtime.
    type Version = Version;
    /// Converts a module to the index of the module in `construct_runtime!`.
    ///
    /// This type is being generated by `construct_runtime!`.
    type PalletInfo = PalletInfo;
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// The data to be stored in an account.
    // type AccountData = ();
    type AccountData = AccountData<u128>;
    // type AccountData = AccountData<u128>;
    
    // type AccountData = AccountData<u128>;

    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = ();
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
    /// The set code logic, just the default since we're not a parachain.
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

// --------------------------------------
// CONSENSUS RELATED FRAME PALLETS
// --------------------------------------
// Notes:
// Aura is the consensus algorithm used for block production.
// Grandpa is the consensus algorithm used for block finalization.
// We want to support multiple flavors of consensus algorithms.
// Specifically we want to implement some proposals defined in the Starknet community forum.
// For more information see: https://community.starknet.io/t/starknet-decentralized-protocol-i-introduction/2671
// You can also follow this issue on github: https://github.com/keep-starknet-strange/madara/issues/83

/// Authority-based consensus protocol used for block production.
/// TODO: Comment and explain the rationale behind the configuration items.
impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = ConstU32<32>;
    type AllowMultipleBlocksPerSlot = ConstBool<false>;
}

/// Deterministic finality mechanism used for block finalization.
/// TODO: Comment and explain the rationale behind the configuration items.
impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;

    type WeightInfo = ();
    type MaxAuthorities = ConstU32<32>;
    type MaxSetIdSessionEntries = ConstU64<0>;
    type MaxNominators = ConstU32<1000>;

    type KeyOwnerProof = sp_core::Void;
    type EquivocationReportSystem = ();
}

/// --------------------------------------
/// OTHER 3RD PARTY FRAME PALLETS
/// --------------------------------------

/// Timestamp manipulation.
/// For instance, we need it to set the timestamp of the Starknet block.
impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = ConsensusOnTimestampSet<Self>;
    type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
    type WeightInfo = ();
}

fn get_execution_constants() -> Arc<VersionedConstants> {
    EXECUTION_CONSTANTS.deref().clone()
}

parameter_types! {
    pub const UnsignedPriority: u64 = 1 << 20;
    pub const TransactionLongevity: u64 = u64::MAX;
    pub const ProtocolVersion: u8 = 0;
    pub const ProgramHash: Felt252Wrapper = SN_OS_PROGRAM_HASH;
    pub ExecutionConstants: Arc<VersionedConstants> = get_execution_constants();
}

/// Implement the OnTimestampSet trait to override the default Aura.
/// This is needed to suppress Aura validations in case of non-default sealing.
pub struct ConsensusOnTimestampSet<T>(PhantomData<T>);
impl<T: pallet_aura::Config> OnTimestampSet<T::Moment> for ConsensusOnTimestampSet<T> {
    fn on_timestamp_set(moment: T::Moment) {
        if Sealing::get() != SealingMode::Default {
            return;
        }
        <pallet_aura::Pallet<T> as OnTimestampSet<T::Moment>>::on_timestamp_set(moment)
    }
}


parameter_types! {
    // NOstr
    pub const UnsignedPriorityNostr:u64= u64::MAX;
    pub const UnsignedInterval: u32=u32::MIN;
    pub const GracePeriod: u32=u32::MIN;
    pub const MaxPrices: u32 =u32::MAX;
}
/// Payload data to be signed when making signed transaction from off-chain workers,
///   inside `create_transaction` function.
// pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;

impl pallet_nostr::Config for Runtime {
    type AuthorityId = pallet_nostr::crypto::TestAuthId;
    type RuntimeEvent = RuntimeEvent;
    type UnsignedPriority = UnsignedPriorityNostr;
    type UnsignedInterval = UnsignedInterval;
    type GracePeriod = GracePeriod;
    type MaxPrices = MaxPrices;
    // type AccountId = AccountId ;
    // type Balance = Balance;
    // type MinBalance = Balance;

}
impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
    RuntimeCall: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: RuntimeCall,
        public: <Signature as traits::Verify>::Signer,
        account: AccountId,
        // nonce: Nonce,
        nonce: u32,
    ) -> Option<(
        RuntimeCall,
        <UncheckedExtrinsic as traits::Extrinsic>::SignaturePayload,
    )> {
        // let tip = 0;

        // take the biggest period possible.
        let period = BlockHashCount::get()
            .checked_next_power_of_two()
            .map(|c| c / 2)
            .unwrap_or(2) as u64;
        // The `System::block_number` is initialized with `n+1`,
        // so the actual block number is `n`.
        // .saturating_sub(1);
        let current_block = (System::block_number() -1) as u64;
        let era = Era::mortal(period, current_block);
        let extra = (
            frame_system::CheckNonZeroSender::<Runtime>::new(),
            frame_system::CheckSpecVersion::<Runtime>::new(),
            frame_system::CheckTxVersion::<Runtime>::new(),
            frame_system::CheckGenesis::<Runtime>::new(),
            frame_system::CheckEra::<Runtime>::from(era),
            frame_system::CheckNonce::<Runtime>::from(nonce),
            frame_system::CheckWeight::<Runtime>::new(),
            // TODO add tx payment
            // pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
        );
        let raw_payload = SignedPayload::new(call, extra)
            .map_err(|e| {
                log::warn!("Unable to create signed payload: {:?}", e);
            })
            .ok()?;
        let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
        // let address = Indices::unlookup(account);
        let address = account;
        let (call, extra, _) = raw_payload.deconstruct();
        // Some((call, (address, signature, extra)))
        Some((call, (sp_runtime::MultiAddress::Id(address), signature, extra)))
    }
}

impl frame_system::offchain::SigningTypes for Runtime {
    type Public = <Signature as traits::Verify>::Signer;
    type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    RuntimeCall: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = RuntimeCall;
}


impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type MaxHolds = ();
}

// impl pallet_balances::Config for Runtime {
//     type MaxLocks = ConstU32<50>;
//     type MaxReserves = ();
//     type ReserveIdentifier = [u8; 8];
//     /// The type for recording an account's balance.
//     type Balance = Balance;
//     /// The ubiquitous event type.
//     type RuntimeEvent = RuntimeEvent;
//     type DustRemoval = ();
//     type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
//     type AccountStore = System;
//     type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
//     type FreezeIdentifier = ();
//     type MaxFreezes = ();
//     type RuntimeHoldReason = ();
//     type RuntimeFreezeReason = ();
//     type MaxHolds = ();
// }

