//! The substrate runtime for the Humanode network.

#![recursion_limit = "256"]
// TODO(#66): switch back to warn
#![allow(missing_docs, clippy::missing_docs_in_private_items)]
// Either generate code at stadard mode, or `no_std`, based on the `std` feature presence.
#![cfg_attr(not(feature = "std"), no_std)]

// If we're in standard compilation mode, embed the build-script generated code that pulls in
// the WASM portion of the runtime, so that it is invocable from the native (aka host) side code.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

// A few exports that help ease life for downstream crates.
use codec::{alloc::string::ToString, Decode, Encode, MaxEncodedLen};
use fp_rpc::TransactionStatus;
pub use frame_support::{
    construct_runtime, parameter_types,
    traits::{
        ConstBool, ConstU128, ConstU16, ConstU32, ConstU64, ConstU8, FindAuthor, Get,
        KeyOwnerProofSystem, Randomness,
    },
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        IdentityFee, Weight,
    },
    ConsensusEngineId, PalletId, StorageValue, WeakBoundedVec,
};
use keystore_bioauth_account_id::KeystoreBioauthAccountId;
pub use pallet_balances::Call as BalancesCall;
use pallet_bioauth::AuthTicket;
use pallet_ethereum::{Call::transact, Transaction as EthereumTransaction};
use pallet_evm::FeeCalculator;
use pallet_evm::{Account as EVMAccount, EnsureAddressTruncated, HashedAddressMapping, Runner};
use pallet_grandpa::{
    fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical as pallet_session_historical;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::CurrencyAdapter;
use primitives_auth_ticket::OpaqueAuthTicket;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_api::impl_runtime_apis;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata, H160, H256, U256};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{
        AccountIdLookup, BlakeTwo256, Block as BlockT, DispatchInfoOf, Dispatchable,
        IdentifyAccount, NumberFor, OpaqueKeys, PostDispatchInfoOf, StaticLookup, Verify,
    },
    transaction_validity::{
        TransactionPriority, TransactionSource, TransactionValidity, TransactionValidityError,
    },
    ApplyExtrinsicResult, MultiSignature, SaturatedConversion,
};
pub use sp_runtime::{Perbill, Permill};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

mod frontier_precompiles;
use frontier_precompiles::FrontierPrecompiles;

mod display_moment;
pub mod eip712;
mod find_author;
pub mod fixed_supply;
pub mod robonode;
#[cfg(test)]
mod tests;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Consensus identity used to tie the consensus signatures to the bioauth identity
/// via session pallet's key ownership logic.
pub type BioauthConsensusId = BabeId;

/// The bioauth identity of a human.
pub type BioauthId = AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    use super::*;

    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;

    impl_opaque_keys! {
        pub struct SessionKeys {
            pub babe: Babe,
            pub grandpa: Grandpa,
            pub im_online: ImOnline,
        }
    }
}

// https://substrate.dev/docs/en/knowledgebase/runtime/upgrades#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("humanode"),
    impl_name: create_runtime_str!("humanode"),
    authoring_version: 1,
    // The version of the runtime specification. A full node will not attempt to use its native
    //   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
    //   `spec_version`, and `authoring_version` are the same between Wasm and native.
    // This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
    //   the compatible custom types.
    spec_version: 100,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

// 1 in 4 blocks (on average, not counting collisions) will be primary BABE blocks.
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

/// The BABE epoch configuration at genesis.
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
    sp_consensus_babe::BabeEpochConfiguration {
        c: PRIMARY_PROBABILITY,
        allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
    };

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const SECS_PER_BLOCK: u64 = MILLISECS_PER_BLOCK / 1000;

// These time units are defined in number of blocks.
pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;
pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 10 * MINUTES;
// NOTE: Currently it is not possible to change the epoch duration after the chain has started.
//       Attempting to do so will brick block production.
pub const EPOCH_DURATION_IN_SLOTS: u64 = {
    const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

    (EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
};

/// The longevity, in blocks, that an equivocation report is valid for.
const REPORT_LONGEVITY: u64 = 3 * EPOCH_DURATION_IN_BLOCKS as u64;

// Consensus related constants.
pub const MAX_AUTHENTICATIONS: u32 = 20 * 1024;
pub const MAX_AUTHORITIES: u32 = MAX_AUTHENTICATIONS;
pub const MAX_NONCES: u32 = 2000 * MAX_AUTHENTICATIONS;

// ImOnline related constants.
// TODO(#311): set proper values
pub const MAX_KEYS: u32 = 20 * 1024;
pub const MAX_PEER_IN_HEARTBEATS: u32 = 3 * MAX_KEYS;
pub const MAX_PEER_DATA_ENCODING_SIZE: u32 = 1_000;

// Constants conditions.
static_assertions::const_assert!(MAX_KEYS >= MAX_AUTHENTICATIONS);
static_assertions::const_assert!(MAX_PEER_IN_HEARTBEATS >= 3 * MAX_AUTHENTICATIONS);

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
    pub const Version: RuntimeVersion = VERSION;
    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
        ::with_sensible_defaults(2 * WEIGHT_PER_SECOND, NORMAL_DISPATCH_RATIO);
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
}

// Configure FRAME pallets to include in runtime.

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
    type Call = Call;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = AccountIdLookup<AccountId, ()>;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = ConstU32<2400>;
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
    type AccountData = pallet_balances::AccountData<Balance>;
    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = ();
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = ConstU16<42>;
    /// The set code logic, just the default since we're not a parachain.
    type OnSetCode = ();
    /// The maximum number of consumers allowed on a single account.
    type MaxConsumers = ConstU32<16>;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

impl pallet_babe::Config for Runtime {
    type EpochDuration = ConstU64<EPOCH_DURATION_IN_SLOTS>;
    type ExpectedBlockTime = ConstU64<MILLISECS_PER_BLOCK>;
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;
    type DisabledValidators = Session;

    type KeyOwnerProofSystem = Historical;

    type KeyOwnerProof =
        <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, BabeId)>>::Proof;

    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        BabeId,
    )>>::IdentificationTuple;

    type HandleEquivocation = pallet_babe::EquivocationHandler<
        Self::KeyOwnerIdentification,
        Offences,
        ConstU64<REPORT_LONGEVITY>,
    >;

    type WeightInfo = ();
    type MaxAuthorities = ConstU32<MAX_AUTHORITIES>;
}

/// A link between the [`AccountId`] as in what we use to sign extrinsics in the system
/// to the [`BioauthId`] as in what we use to identify to the robonode and tie the biometrics to.
pub struct IdentityValidatorIdOf;
impl sp_runtime::traits::Convert<AccountId, Option<BioauthId>> for IdentityValidatorIdOf {
    fn convert(account_id: AccountId) -> Option<BioauthId> {
        Some(account_id)
    }
}

impl pallet_session::Config for Runtime {
    type Event = Event;
    type ValidatorId = BioauthId;
    type ValidatorIdOf = IdentityValidatorIdOf;
    type ShouldEndSession = Babe;
    type NextSessionRotation = Babe;
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, HumanodeSession>;
    type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type Keys = opaque::SessionKeys;
    type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

impl pallet_session::historical::Config for Runtime {
    type FullIdentification = pallet_humanode_session::IdentificationFor<Self>;
    type FullIdentificationOf = pallet_humanode_session::CurrentSessionIdentificationOf<Self>;
}

impl pallet_grandpa::Config for Runtime {
    type Event = Event;
    type Call = Call;

    type KeyOwnerProofSystem = Historical;

    type KeyOwnerProof =
        <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        GrandpaId,
    )>>::IdentificationTuple;

    type HandleEquivocation = pallet_grandpa::EquivocationHandler<
        Self::KeyOwnerIdentification,
        Offences,
        ConstU64<REPORT_LONGEVITY>,
    >;

    type WeightInfo = ();
    type MaxAuthorities = ConstU32<MAX_AUTHORITIES>;
}

/// A timestamp: milliseconds since the unix epoch.
pub type UnixMilliseconds = u64;

impl pallet_timestamp::Config for Runtime {
    type Moment = UnixMilliseconds;
    type OnTimestampSet = Babe;
    type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
    type WeightInfo = ();
}

impl pallet_authorship::Config for Runtime {
    type FindAuthor = find_author::FindAuthorFromSession<find_author::FindAuthorBabe, BabeId>;
    type UncleGenerations = ConstU32<5>;
    type FilterUncle = ();
    type EventHandler = (ImOnline,);
}

parameter_types! {
    pub const TreasuryPotPalletId: PalletId = PalletId(*b"hmnd/tr1");
    pub const FeesPotPalletId: PalletId = PalletId(*b"hmnd/fe1");
}

type PotInstanceTreasury = pallet_pot::Instance1;
type PotInstanceFees = pallet_pot::Instance2;

impl pallet_pot::Config<PotInstanceTreasury> for Runtime {
    type Event = Event;
    type PalletId = TreasuryPotPalletId;
    type Currency = fixed_supply::Currency;
}

impl pallet_pot::Config<PotInstanceFees> for Runtime {
    type Event = Event;
    type PalletId = FeesPotPalletId;
    type Currency = fixed_supply::Currency;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type Event = Event;
    /// The dust is collected at the treasury pot.
    /// Regardless of the existential deposit value, this will never alter the total issuance.
    type DustRemoval = fixed_supply::ImbalanceAdapterHanlder<
        pallet_balances::NegativeImbalance<Self>,
        fixed_supply::NegativeImbalance,
        TreasuryPot,
    >;
    type ExistentialDeposit = ConstU128<500>;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction = CurrencyAdapter<fixed_supply::Currency, FeesPot>;
    type OperationalFeeMultiplier = ConstU8<5>;
    type WeightToFee = IdentityFee<Balance>;
    type LengthToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ();
}

impl pallet_sudo::Config for Runtime {
    type Event = Event;
    type Call = Call;
}

pub struct PrimitiveAuthTicketConverter;

pub enum PrimitiveAuthTicketConverterError {
    Ticket(codec::Error),
    PublicKey(()),
}

impl pallet_bioauth::TryConvert<OpaqueAuthTicket, pallet_bioauth::AuthTicket<BioauthId>>
    for PrimitiveAuthTicketConverter
{
    type Error = PrimitiveAuthTicketConverterError;

    fn try_convert(
        value: OpaqueAuthTicket,
    ) -> Result<pallet_bioauth::AuthTicket<BioauthId>, Self::Error> {
        let primitives_auth_ticket::AuthTicket {
            public_key,
            authentication_nonce: nonce,
        } = (&value)
            .try_into()
            .map_err(PrimitiveAuthTicketConverterError::Ticket)?;

        let public_key = public_key
            .as_slice()
            .try_into()
            .map_err(PrimitiveAuthTicketConverterError::PublicKey)?;

        Ok(AuthTicket { public_key, nonce })
    }
}

pub struct CurrentMoment;

impl pallet_bioauth::CurrentMoment<UnixMilliseconds> for CurrentMoment {
    fn now() -> UnixMilliseconds {
        pallet_timestamp::Pallet::<Runtime>::now()
    }
}

const TIMESTAMP_SECOND: UnixMilliseconds = 1000;
const TIMESTAMP_MINUTE: UnixMilliseconds = 60 * TIMESTAMP_SECOND;
const TIMESTAMP_HOUR: UnixMilliseconds = 60 * TIMESTAMP_MINUTE;
const TIMESTAMP_DAY: UnixMilliseconds = 24 * TIMESTAMP_HOUR;
const AUTHENTICATIONS_EXPIRE_AFTER: UnixMilliseconds = 7 * TIMESTAMP_DAY;

impl pallet_bioauth::Config for Runtime {
    type Event = Event;
    type RobonodePublicKey = robonode::PublicKey;
    type RobonodeSignature = Vec<u8>;
    type ValidatorPublicKey = BioauthId;
    type OpaqueAuthTicket = primitives_auth_ticket::OpaqueAuthTicket;
    type AuthTicketCoverter = PrimitiveAuthTicketConverter;
    type ValidatorSetUpdater = ();
    type Moment = UnixMilliseconds;
    type DisplayMoment = display_moment::DisplayMoment;
    type CurrentMoment = CurrentMoment;
    type AuthenticationsExpireAfter = ConstU64<AUTHENTICATIONS_EXPIRE_AFTER>;
    type WeightInfo = pallet_bioauth::weights::SubstrateWeight<Runtime>;
    type MaxAuthentications = ConstU32<MAX_AUTHENTICATIONS>;
    type MaxNonces = ConstU32<MAX_NONCES>;
    type BeforeAuthHook = ();
    type AfterAuthHook = ();
}

#[cfg(feature = "runtime-benchmarks")]
impl pallet_bioauth::benchmarking::AuthTicketBuilder for Runtime {
    fn build(
        public_key: Vec<u8>,
        authentication_nonce: Vec<u8>,
    ) -> <Self as pallet_bioauth::Config>::OpaqueAuthTicket {
        OpaqueAuthTicket::from(&primitives_auth_ticket::AuthTicket {
            public_key,
            authentication_nonce,
        })
    }
}

impl pallet_bootnodes::Config for Runtime {
    type BootnodeId = AccountId;
    type MaxBootnodes = ConstU32<16>;
}

parameter_types! {
    pub MaxSessionValidators: u32 = <<Runtime as pallet_bootnodes::Config>::MaxBootnodes as Get<u32>>::get() + <<Runtime as pallet_bioauth::Config>::MaxAuthentications as Get<u32>>::get();
}

impl pallet_humanode_session::Config for Runtime {
    type ValidatorPublicKeyOf = IdentityValidatorIdOf;
    type BootnodeIdOf = sp_runtime::traits::Identity;
    type MaxSessionValidators = MaxSessionValidators;
}

pub struct OffenceSlasher;

impl
    sp_staking::offence::OnOffenceHandler<
        AccountId,
        pallet_im_online::IdentificationTuple<Runtime>,
        Weight,
    > for OffenceSlasher
{
    fn on_offence(
        offenders: &[sp_staking::offence::OffenceDetails<
            AccountId,
            pallet_im_online::IdentificationTuple<Runtime>,
        >],
        _slash_fraction: &[Perbill],
        _session: sp_staking::SessionIndex,
        disable_strategy: sp_staking::offence::DisableStrategy,
    ) -> Weight {
        if disable_strategy == sp_staking::offence::DisableStrategy::Never {
            return 0;
        }
        let mut weight: Weight = 0;
        let weights = <Runtime as frame_system::Config>::DbWeight::get();
        for details in offenders {
            let (_offender, identity) = &details.offender;
            match identity {
                pallet_humanode_session::Identification::Bioauth(authentication) => {
                    let has_deathenticated = Bioauth::deauthenticate(&authentication.public_key);
                    weight = weight.saturating_add(
                        weights.reads_writes(1, if has_deathenticated { 1 } else { 0 }),
                    );
                }
                pallet_humanode_session::Identification::Bootnode(..) => {
                    // Never slash the bootnodes.
                }
            }
        }
        weight
    }
}

impl pallet_im_online::Config for Runtime {
    type AuthorityId = ImOnlineId;
    type Event = Event;
    type NextSessionRotation = Babe;
    type ValidatorSet = Historical;
    type ReportUnresponsiveness = Offences;
    type UnsignedPriority = ConstU64<{ TransactionPriority::MAX }>;
    type WeightInfo = pallet_im_online::weights::SubstrateWeight<Runtime>;
    type MaxKeys = ConstU32<MAX_KEYS>;
    type MaxPeerInHeartbeats = ConstU32<MAX_PEER_IN_HEARTBEATS>;
    type MaxPeerDataEncodingSize = ConstU32<MAX_PEER_DATA_ENCODING_SIZE>;
}

impl pallet_offences::Config for Runtime {
    type Event = Event;
    type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
    type OnOffenceHandler = OffenceSlasher;
}

parameter_types! {
    pub BlockGasLimit: U256 = U256::from(u32::max_value());
    pub PrecompilesValue: FrontierPrecompiles<Runtime> = FrontierPrecompiles::<_>::default();
}

impl pallet_evm::Config for Runtime {
    type FeeCalculator = BaseFee;
    type GasWeightMapping = ();
    type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
    type CallOrigin = EnsureAddressTruncated;
    type WithdrawOrigin = EnsureAddressTruncated;
    type AddressMapping = HashedAddressMapping<BlakeTwo256>;
    type Currency = fixed_supply::Currency;
    type Event = Event;
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    type PrecompilesType = FrontierPrecompiles<Self>;
    type PrecompilesValue = PrecompilesValue;
    type ChainId = EthereumChainId;
    type BlockGasLimit = BlockGasLimit;
    type OnChargeTransaction = pallet_evm::EVMCurrencyAdapter<fixed_supply::Currency, FeesPot>;
    type FindAuthor = find_author::FindAuthorTruncated<
        find_author::FindAuthorFromSession<find_author::FindAuthorBabe, BabeId>,
    >;
}

impl pallet_ethereum::Config for Runtime {
    type Event = Event;
    type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
}

parameter_types! {
    pub BoundDivision: U256 = U256::from(1024);
}

impl pallet_dynamic_fee::Config for Runtime {
    type MinGasPriceBoundDivisor = BoundDivision;
}

parameter_types! {
    pub DefaultBaseFeePerGas: U256 = U256::from(1_000_000_000);
}

pub struct BaseFeeThreshold;
impl pallet_base_fee::BaseFeeThreshold for BaseFeeThreshold {
    fn lower() -> Permill {
        Permill::zero()
    }
    fn ideal() -> Permill {
        Permill::from_parts(500_000)
    }
    fn upper() -> Permill {
        Permill::from_parts(1_000_000)
    }
}

impl pallet_base_fee::Config for Runtime {
    type Event = Event;
    type Threshold = BaseFeeThreshold;
    type IsActive = ConstBool<true>;
    type DefaultBaseFeePerGas = DefaultBaseFeePerGas;
}

impl pallet_ethereum_chain_id::Config for Runtime {}

impl pallet_evm_accounts_mapping::Config for Runtime {
    type Event = Event;
    type Verifier = eip712::AccountClaimVerifier;
}

// Create the runtime by composing the FRAME pallets that were previously
// configured.
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system,
        RandomnessCollectiveFlip: pallet_randomness_collective_flip,
        Timestamp: pallet_timestamp,
        Bootnodes: pallet_bootnodes,
        Bioauth: pallet_bioauth,
        Babe: pallet_babe,
        // Authorship must be before other pallets that rely on the data it captures.
        Authorship: pallet_authorship,
        Balances: pallet_balances,
        TreasuryPot: pallet_pot::<Instance1>,
        FeesPot: pallet_pot::<Instance2>,
        TransactionPayment: pallet_transaction_payment,
        Session: pallet_session,
        Offences: pallet_offences,
        Historical: pallet_session_historical,
        HumanodeSession: pallet_humanode_session,
        EthereumChainId: pallet_ethereum_chain_id,
        Sudo: pallet_sudo,
        Grandpa: pallet_grandpa,
        Ethereum: pallet_ethereum,
        EVM: pallet_evm,
        DynamicFee: pallet_dynamic_fee,
        BaseFee: pallet_base_fee,
        ImOnline: pallet_im_online,
        EvmAccountsMapping: pallet_evm_accounts_mapping,
    }
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_bioauth::CheckBioauthTx<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    fp_self_contained::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

impl frame_system::offchain::CreateSignedTransaction<Call> for Runtime {
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        public: <Signature as sp_runtime::traits::Verify>::Signer,
        account: AccountId,
        nonce: Index,
    ) -> Option<(
        Call,
        <UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
    )> {
        let tip = 0;
        // take the biggest period possible.
        let period = <Self::BlockHashCount as Get<Self::BlockNumber>>::get()
            .checked_next_power_of_two()
            .map(|c| c / 2)
            .unwrap_or(2) as u64;
        let current_block = System::block_number()
            .saturated_into::<u64>()
            // The `System::block_number` is initialized with `n+1`,
            // so the actual block number is `n`.
            .saturating_sub(1);
        let era = sp_runtime::generic::Era::mortal(period, current_block);
        let extra = (
            frame_system::CheckSpecVersion::<Runtime>::new(),
            frame_system::CheckTxVersion::<Runtime>::new(),
            frame_system::CheckGenesis::<Runtime>::new(),
            frame_system::CheckEra::<Runtime>::from(era),
            frame_system::CheckNonce::<Runtime>::from(nonce),
            frame_system::CheckWeight::<Runtime>::new(),
            pallet_bioauth::CheckBioauthTx::<Runtime>::new(),
            pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
        );
        let raw_payload = SignedPayload::new(call, extra).ok()?;
        let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
        let address = AccountIdLookup::unlookup(account);
        let (call, extra, _) = raw_payload.deconstruct();
        Some((call, (address, signature, extra)))
    }
}

impl frame_system::offchain::SigningTypes for Runtime {
    type Public = <Signature as sp_runtime::traits::Verify>::Signer;
    type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    Call: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = Call;
}

impl fp_self_contained::SelfContainedCall for Call {
    type SignedInfo = H160;

    fn is_self_contained(&self) -> bool {
        match self {
            Call::Ethereum(call) => call.is_self_contained(),
            _ => false,
        }
    }

    fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
        match self {
            Call::Ethereum(call) => call.check_self_contained(),
            _ => None,
        }
    }

    fn validate_self_contained(
        &self,
        info: &Self::SignedInfo,
        dispatch_info: &DispatchInfoOf<Call>,
        len: usize,
    ) -> Option<TransactionValidity> {
        match self {
            Call::Ethereum(call) => call.validate_self_contained(info, dispatch_info, len),
            _ => None,
        }
    }

    fn pre_dispatch_self_contained(
        &self,
        info: &Self::SignedInfo,
    ) -> Option<Result<(), TransactionValidityError>> {
        match self {
            Call::Ethereum(call) => call.pre_dispatch_self_contained(info),
            _ => None,
        }
    }

    fn apply_self_contained(
        self,
        info: Self::SignedInfo,
    ) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
        match self {
            call @ Call::Ethereum(pallet_ethereum::Call::transact { .. }) => Some(call.dispatch(
                Origin::from(pallet_ethereum::RawOrigin::EthereumTransaction(info)),
            )),
            _ => None,
        }
    }
}

pub struct TransactionConverter;

impl fp_rpc::ConvertTransaction<UncheckedExtrinsic> for TransactionConverter {
    fn convert_transaction(&self, transaction: pallet_ethereum::Transaction) -> UncheckedExtrinsic {
        UncheckedExtrinsic::new_unsigned(
            pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
        )
    }
}

impl fp_rpc::ConvertTransaction<opaque::UncheckedExtrinsic> for TransactionConverter {
    fn convert_transaction(
        &self,
        transaction: pallet_ethereum::Transaction,
    ) -> opaque::UncheckedExtrinsic {
        let extrinsic = UncheckedExtrinsic::new_unsigned(
            pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
        );
        let encoded = extrinsic.encode();
        opaque::UncheckedExtrinsic::decode(&mut &encoded[..])
            .expect("Encoded extrinsic is always valid")
    }
}

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) ->
            Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl author_ext_api::AuthorExtApi<Block, KeystoreBioauthAccountId> for Runtime {
        fn create_signed_set_keys_extrinsic(
            id: &KeystoreBioauthAccountId,
            session_keys: Vec<u8>
        ) -> Result<<Block as BlockT>::Extrinsic, author_ext_api::CreateSignedSetKeysExtrinsicError> {
            let account_id =
                AccountId::new(<KeystoreBioauthAccountId as sp_application_crypto::AppKey>::UntypedGeneric::from(id.clone()).0);
            let public_id = <KeystoreBioauthAccountId as frame_system::offchain::AppCrypto<
                    <Runtime as frame_system::offchain::SigningTypes>::Public,
                    <Runtime as frame_system::offchain::SigningTypes>::Signature
                >>::GenericPublic::from(id.clone());

            let keys = <Runtime as pallet_session::Config>::Keys::decode(&mut session_keys.as_slice())
                .map_err(|err| author_ext_api::CreateSignedSetKeysExtrinsicError::SessionKeysDecoding(err.to_string()))?;
            let session_call = pallet_session::Call::set_keys::<Runtime> { keys, proof: vec![] };
            let (call, (address, signature, extra)) =
                <Runtime as frame_system::offchain::CreateSignedTransaction<Call>>::create_transaction::<KeystoreBioauthAccountId>(
                    session_call.into(),
                    public_id.into(),
                    account_id.clone(),
                    System::account_nonce(account_id),
                ).ok_or(author_ext_api::CreateSignedSetKeysExtrinsicError::SignedExtrinsicCreation)?;

            Ok(<Block as BlockT>::Extrinsic::new_signed(call, address, signature, extra))
        }
    }

    impl bioauth_flow_api::BioauthFlowApi<Block, KeystoreBioauthAccountId, UnixMilliseconds> for Runtime {
        fn bioauth_status(id: &KeystoreBioauthAccountId) -> bioauth_flow_api::BioauthStatus<UnixMilliseconds> {
            let id =
                AccountId::new(<KeystoreBioauthAccountId as sp_application_crypto::AppKey>::UntypedGeneric::from(id.clone()).0);
            let active_authentications = Bioauth::active_authentications().into_inner();
            let maybe_active_authentication = active_authentications
                .iter()
                .find(|stored_public_key| stored_public_key.public_key == id);
            match maybe_active_authentication {
                None => bioauth_flow_api::BioauthStatus::Inactive,
                Some(v) => bioauth_flow_api::BioauthStatus::Active {
                    expires_at: v.expires_at,
                },
            }
        }

        fn create_authenticate_extrinsic(
            auth_ticket: Vec<u8>,
            auth_ticket_signature: Vec<u8>
        ) -> <Block as BlockT>::Extrinsic {
            let authenticate = pallet_bioauth::Authenticate {
                ticket: auth_ticket.into(),
                ticket_signature: auth_ticket_signature,
            };

            let call = pallet_bioauth::Call::authenticate { req: authenticate };

            <Block as BlockT>::Extrinsic::new_unsigned(call.into())
        }
    }

    impl fp_rpc::ConvertTransactionRuntimeApi<Block> for Runtime {
        fn convert_transaction(transaction: EthereumTransaction) -> <Block as BlockT>::Extrinsic {
            UncheckedExtrinsic::new_unsigned(
                pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
            )
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            opaque::SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn current_set_id() -> fg_primitives::SetId {
            Grandpa::current_set_id()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Grandpa::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((fg_primitives::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(fg_primitives::OpaqueKeyOwnershipProof::new)
        }
    }


    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
            // The choice of `c` parameter (where `1 - c` represents the
            // probability of a slot being empty), is done in accordance to the
            // slot duration and expected target block time, for safely
            // resisting network delays of maximum two seconds.
            // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
            sp_consensus_babe::BabeGenesisConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: <Self as pallet_babe::Config>::EpochDuration::get(),
                c: BABE_GENESIS_EPOCH_CONFIG.c,
                genesis_authorities: Babe::authorities().to_vec(),
                randomness: Babe::randomness(),
                allowed_slots: BABE_GENESIS_EPOCH_CONFIG.allowed_slots,
            }
        }

        fn current_epoch_start() -> sp_consensus_babe::Slot {
            Babe::current_epoch_start()
        }

        fn current_epoch() -> sp_consensus_babe::Epoch {
            Babe::current_epoch()
        }

        fn next_epoch() -> sp_consensus_babe::Epoch {
            Babe::next_epoch()
        }

        fn generate_key_ownership_proof(
            _slot: sp_consensus_babe::Slot,
            authority_id: sp_consensus_babe::AuthorityId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Babe::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
        fn chain_id() -> u64 {
            <Runtime as pallet_evm::Config>::ChainId::get()
        }

        fn account_basic(address: H160) -> EVMAccount {
            let (account, _) = EVM::account_basic(&address);
            account
        }

        fn gas_price() -> U256 {
            let (gas_price, _) = <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price();
            gas_price
        }

        fn account_code_at(address: H160) -> Vec<u8> {
            EVM::account_codes(address)
        }

        fn author() -> H160 {
            <pallet_evm::Pallet<Runtime>>::find_author()
        }

        fn storage_at(address: H160, index: U256) -> H256 {
            let mut tmp = [0u8; 32];
            index.to_big_endian(&mut tmp);
            EVM::account_storages(address, H256::from_slice(&tmp[..]))
        }

        fn call(
            from: H160,
            to: H160,
            data: Vec<u8>,
            value: U256,
            gas_limit: U256,
            max_fee_per_gas: Option<U256>,
            max_priority_fee_per_gas: Option<U256>,
            nonce: Option<U256>,
            estimate: bool,
            access_list: Option<Vec<(H160, Vec<H256>)>>,
        ) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
            let config = if estimate {
                let mut config = <Runtime as pallet_evm::Config>::config().clone();
                config.estimate = true;
                config
            } else {
                <Runtime as pallet_evm::Config>::config().clone()
            };

            let is_transactional = false;
            <Runtime as pallet_evm::Config>::Runner::call(
                from,
                to,
                data,
                value,
                gas_limit.low_u64(),
                max_fee_per_gas,
                max_priority_fee_per_gas,
                nonce,
                access_list.unwrap_or_default(),
                is_transactional,
                &config,
            ).map_err(|err| err.error.into())
        }

        fn create(
            from: H160,
            data: Vec<u8>,
            value: U256,
            gas_limit: U256,
            max_fee_per_gas: Option<U256>,
            max_priority_fee_per_gas: Option<U256>,
            nonce: Option<U256>,
            estimate: bool,
            access_list: Option<Vec<(H160, Vec<H256>)>>,
        ) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
            let config = if estimate {
                let mut config = <Runtime as pallet_evm::Config>::config().clone();
                config.estimate = true;
                config
            } else {
                <Runtime as pallet_evm::Config>::config().clone()
            };

            let is_transactional = false;
            <Runtime as pallet_evm::Config>::Runner::create(
                from,
                data,
                value,
                gas_limit.low_u64(),
                max_fee_per_gas,
                max_priority_fee_per_gas,
                nonce,
                access_list.unwrap_or_default(),
                is_transactional,
                &config,
            ).map_err(|err| err.error.into())
        }

        fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
            Ethereum::current_transaction_statuses()
        }

        fn current_block() -> Option<pallet_ethereum::Block> {
            Ethereum::current_block()
        }

        fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
            Ethereum::current_receipts()
        }

        fn current_all() -> (
            Option<pallet_ethereum::Block>,
            Option<Vec<pallet_ethereum::Receipt>>,
            Option<Vec<TransactionStatus>>
        ) {
            (
                Ethereum::current_block(),
                Ethereum::current_receipts(),
                Ethereum::current_transaction_statuses()
            )
        }

        fn extrinsic_filter(
            xts: Vec<<Block as BlockT>::Extrinsic>,
        ) -> Vec<EthereumTransaction> {
            xts.into_iter().filter_map(|xt| match xt.0.function {
                Call::Ethereum(transact { transaction }) => Some(transaction),
                _ => None
            }).collect::<Vec<EthereumTransaction>>()
        }

        fn elasticity() -> Option<Permill> {
            Some(BaseFee::elasticity())
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
        for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{list_benchmark, baseline, Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            let mut list = Vec::<BenchmarkList>::new();

            list_benchmark!(list, extra, frame_benchmarking, BaselineBench::<Runtime>);
            list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
            list_benchmark!(list, extra, pallet_balances, Balances);
            list_benchmark!(list, extra, pallet_timestamp, Timestamp);
            list_benchmark!(list, extra, pallet_bioauth, Bioauth);

            let storage_info = AllPalletsWithSystem::storage_info();

            (list, storage_info)
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{Benchmarking, baseline, BenchmarkBatch, add_benchmark, TrackedStorageKey};

            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            impl frame_system_benchmarking::Config for Runtime {}
            impl baseline::Config for Runtime {}

            let whitelist: Vec<TrackedStorageKey> = vec![
                // Block Number
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac")
                    .to_vec().into(),
                // Total Issuance
                hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80")
                    .to_vec().into(),
                // Execution Phase
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a")
                    .to_vec().into(),
                // Event Count
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850")
                    .to_vec().into(),
                // System Events
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7")
                    .to_vec().into(),
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);

            add_benchmark!(params, batches, frame_benchmarking, BaselineBench::<Runtime>);
            add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
            add_benchmark!(params, batches, pallet_balances, Balances);
            add_benchmark!(params, batches, pallet_timestamp, Timestamp);
            add_benchmark!(params, batches, pallet_bioauth, Bioauth);

            Ok(batches)
        }
    }

    #[cfg(feature = "try-runtime")]
    impl frame_try_runtime::TryRuntime<Block> for Runtime {
        fn on_runtime_upgrade() -> (Weight, Weight) {
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here. If any of the pre/post migration checks fail, we shall stop
            // right here and right now.
            let weight = Executive::try_runtime_upgrade().unwrap();
            (weight, BlockWeights::get().max_block)
        }

        fn execute_block_no_check(block: Block) -> Weight {
            Executive::execute_block_no_check(block)
        }
    }
}
