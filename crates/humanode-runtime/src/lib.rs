//! The substrate runtime for the Humanode network.

#![recursion_limit = "256"]
// TODO(#66): switch back to warn
#![allow(missing_docs, clippy::missing_docs_in_private_items)]
// Either generate code at standard mode, or `no_std`, based on the `std` feature presence.
#![cfg_attr(not(feature = "std"), no_std)]
// Runtime impl macros generate non-snake case names.
#![allow(non_snake_case)]

// If we're in standard compilation mode, embed the build-script generated code that pulls in
// the WASM portion of the runtime, so that it is invocable from the native (aka host) side code.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

// A few exports that help ease life for downstream crates.
use codec::{alloc::string::ToString, Decode, Encode, MaxEncodedLen};
use fp_rpc::TransactionStatus;
use frame_support::traits::LockIdentifier;
pub use frame_support::{
    construct_runtime, parameter_types,
    traits::{
        ConstBool, ConstU128, ConstU16, ConstU32, ConstU64, ConstU8, FindAuthor, Get,
        KeyOwnerProofSystem, OnFinalize, Randomness,
    },
    weights::{
        constants::{
            BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
        },
        Weight,
    },
    ConsensusEngineId, PalletId, StorageValue, WeakBoundedVec,
};
pub use frame_system::Call as SystemCall;
use keystore_bioauth_account_id::KeystoreBioauthAccountId;
pub use pallet_balances::Call as BalancesCall;
use pallet_bioauth::AuthTicket;
use pallet_ethereum::{
    Call::transact, PostLogContent as EthereumPostLogContent, Transaction as EthereumTransaction,
};
use pallet_evm::FeeCalculator;
use pallet_evm::{Account as EVMAccount, Runner};
use pallet_grandpa::{
    fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical as pallet_session_historical;
pub use pallet_timestamp::Call as TimestampCall;
pub use pallet_token_claims as token_claims;
use primitives_auth_ticket::OpaqueAuthTicket;
pub use primitives_ethereum::EthereumAddress;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_api::impl_runtime_apis;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{
    crypto::{AccountId32, KeyTypeId},
    OpaqueMetadata, H160, H256, U256,
};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{
        AccountIdLookup, BlakeTwo256, Block as BlockT, DispatchInfoOf, Dispatchable,
        IdentifyAccount, Identity, NumberFor, One, OpaqueKeys, PostDispatchInfoOf, StaticLookup,
        Verify,
    },
    transaction_validity::{
        TransactionPriority, TransactionSource, TransactionValidity, TransactionValidityError,
    },
    ApplyExtrinsicResult, MultiSignature,
};
pub use sp_runtime::{Perbill, Permill};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

mod frontier_precompiles;
mod vesting;
use frontier_precompiles::{precompiles_constants, FrontierPrecompiles};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod constants;
mod currency_swap;
mod deauthentication_reason;
#[cfg(test)]
mod dev_utils;
mod display_moment;
pub mod eth_sig;
mod find_author;
mod fixed_supply;
pub mod robonode;
#[cfg(test)]
mod tests;
mod weights;

pub mod utils;

pub use constants::{
    babe::{BABE_GENESIS_EPOCH_CONFIG, EPOCH_DURATION_IN_SLOTS, MAX_AUTHORITIES, SLOT_DURATION},
    bioauth::{AUTHENTICATIONS_EXPIRE_AFTER, MAX_AUTHENTICATIONS, MAX_NONCES},
    block_time::MILLISECS_PER_BLOCK,
    equivocation::REPORT_LONGEVITY,
    ethereum::EXTRA_DATA_LENGTH,
    im_online::{MAX_KEYS, MAX_PEER_DATA_ENCODING_SIZE, MAX_PEER_IN_HEARTBEATS},
};
use deauthentication_reason::DeauthenticationReason;
use static_assertions::const_assert;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain.
///
/// We do not define this type via signing scheme because we effectively need this type to be able
/// to hold the values that are beyond the standard signing facilities.
/// Overall, the [`AccountId`] type must be sufficient to hold the following kinds of addresses
/// (non-exhaustive list):
///
/// - all the standard [`Signature`] account identifiers:
///     - the Ed25519 public key
///     - the Sr25519 public key
///     - the Blake2 hash of the compressed ECDSA public key
/// - pot account addresses - they are still addresses but can't be a subject to signing because
///   they don't belong to an asymmetric keypair at all;
/// - multisig account address - this is an address type that is driven by the multisig pallet, and
///   also technically does not correspond to an asymmetric keypair;
/// - EVM addresses - the addresses used by EVM, of which we have to possible variants:
///   - EVM ECDSA addresses - these are the last 20 bytes of the Keccak (`keccak_256`) hash of
///     the uncompressed ECDSA public key, as used in Ethereum; this has nothing to do with
///     the [`sp_runtime::MultiSigner::Ecdsa`] - because that one is built differently, however
///     the underlying asymmetric keypair can be the same. The signatures won't match however,
///     because for the [`MultiSignature::Ecdsa`] variant, [`MultiSignature::verify`] computes
///     the message to verify the signature against in a different way to how Ethereum does it -
///     Substrate uses Blake2 and Ethereum uses Keccak;
///   - EVM code address - this in an address for an EVM smart contract, which can be any 20 bytes
///     really; it is usually constructed using the last 20 bytes of using the Keccak hash somehow
///     obtained from of the contract code, the address of whoever creates the contract and some
///     salt/nonce; note that there are currently multiple simultaneously supported algorithms for
///     how the contract addresses can be constructed by the EVM, and new might be introduced in
///     the future.
///   Both of the 20-byte EVM address types are stored with a certain format in a wider 32-byte
///   [`AccountId32`] type, that enables us to distinguish them from other address types; there's
///   a certain chance that a non-EVM address would just so happen to be matching the EVM address
///   encoding format we use for EVM accounts, which would mess things up.
///
/// We acknowledge that there is largely a limitation of the Substrate's core architecture that does
/// not permit for a more explicit value kind differentiation.
/// Although we can alter the [`AccountId`] to be a more explicit enum, and tweak
/// the [`frame_system::Config::Lookup`] the amount of work at the surrounding ecosystem
/// (i.e. Polkadot.js) is beyond the reasonable effort for us now.
pub type AccountId = AccountId32;

/// Evm account identifier.
pub type EvmAccountId = H160;

// Ensure that the `AccountId` it equivalent to the public key of our transaction signing scheme.
static_assertions::assert_type_eq_all!(
    AccountId,
    <<Signature as Verify>::Signer as IdentifyAccount>::AccountId
);

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

// https://docs.substrate.io/build/upgrade-the-runtime
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
    spec_version: 119,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
const MAX_BLOCK_LENGTH: u32 = 5 * 1024 * 1024;

/// We allow for 2 seconds of compute with a 6 second average block time.
const EXPECTED_BLOCK_WEIGHT: Weight =
    Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2), u64::MAX);

parameter_types! {
    pub const Version: RuntimeVersion = VERSION;
    pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
        ::with_sensible_defaults(EXPECTED_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO);
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(MAX_BLOCK_LENGTH, NORMAL_DISPATCH_RATIO);
    pub SS58Prefix: u16 = ChainProperties::ss58_prefix();
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
    type RuntimeCall = RuntimeCall;
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
    type RuntimeEvent = RuntimeEvent;
    /// The ubiquitous origin type.
    type RuntimeOrigin = RuntimeOrigin;
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
    type SystemWeightInfo = weights::frame_system::WeightInfo<Runtime>;
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    /// The Humande prefix is defined at pallet-chain-properties. It allows us to set up
    /// it easy in genesis before launching the chain without changing the code itself.
    type SS58Prefix = SS58Prefix;
    /// The set code logic, just the default since we're not a parachain.
    type OnSetCode = ();
    /// The maximum number of consumers allowed on a single account.
    type MaxConsumers = ConstU32<16>;
}

impl pallet_babe::Config for Runtime {
    type EpochDuration = ConstU64<EPOCH_DURATION_IN_SLOTS>;
    type ExpectedBlockTime = ConstU64<MILLISECS_PER_BLOCK>;
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;
    type DisabledValidators = Session;

    type KeyOwnerProof =
        <Historical as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::Proof;
    type EquivocationReportSystem = pallet_babe::EquivocationReportSystem<
        Self,
        Offences,
        Historical,
        ConstU64<REPORT_LONGEVITY>,
    >;

    type WeightInfo = (); // TODO(#578): babe weights are broken
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
    type RuntimeEvent = RuntimeEvent;
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
    type RuntimeEvent = RuntimeEvent;

    type KeyOwnerProof = <Historical as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
    type EquivocationReportSystem = pallet_grandpa::EquivocationReportSystem<
        Self,
        Offences,
        Historical,
        ConstU64<REPORT_LONGEVITY>,
    >;

    type WeightInfo = (); // TODO(#578): grandpa weights are broken
    type MaxAuthorities = ConstU32<MAX_AUTHORITIES>;
    type MaxSetIdSessionEntries = ConstU64<REPORT_LONGEVITY>;
}

/// A timestamp: milliseconds since the unix epoch.
pub type UnixMilliseconds = u64;

impl pallet_timestamp::Config for Runtime {
    type Moment = UnixMilliseconds;
    type OnTimestampSet = Babe;
    type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
    type WeightInfo = weights::pallet_timestamp::WeightInfo<Runtime>;
}

impl pallet_chain_start_moment::Config for Runtime {
    type Time = Timestamp;
}

impl pallet_authorship::Config for Runtime {
    type FindAuthor = find_author::FindAuthorFromSession<find_author::FindAuthorBabe, BabeId>;
    type EventHandler = (ImOnline,);
}

parameter_types! {
    pub const TreasuryPotPalletId: PalletId = PalletId(*b"hmnd/tr1");
    pub const FeesPotPalletId: PalletId = PalletId(*b"hmnd/fe1");
    pub const TokenClaimsPotPalletId: PalletId = PalletId(*b"hmnd/tc1");
    pub const NativeToEvmSwapBridgePotPalletId: PalletId = PalletId(*b"hmcs/ne1");
    pub const EvmToNativeSwapBridgePotPalletId: PalletId = PalletId(*b"hmcs/en1");
}

type PotInstanceTreasury = pallet_pot::Instance1;
type PotInstanceFees = pallet_pot::Instance2;
type PotInstanceTokenClaims = pallet_pot::Instance3;
type PotInstanceNativeToEvmSwapBridge = pallet_pot::Instance4;
type PotInstanceEvmToNativeSwapBridge = pallet_pot::Instance5;

impl pallet_pot::Config<PotInstanceTreasury> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = AccountId;
    type PalletId = TreasuryPotPalletId;
    type Currency = Balances;
}

impl pallet_pot::Config<PotInstanceFees> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = AccountId;
    type PalletId = FeesPotPalletId;
    type Currency = Balances;
}

impl pallet_pot::Config<PotInstanceTokenClaims> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = AccountId;
    type PalletId = TokenClaimsPotPalletId;
    type Currency = Balances;
}

impl pallet_pot::Config<PotInstanceNativeToEvmSwapBridge> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = AccountId;
    type PalletId = NativeToEvmSwapBridgePotPalletId;
    type Currency = Balances;
}

impl pallet_pot::Config<PotInstanceEvmToNativeSwapBridge> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = EvmAccountId;
    type PalletId = EvmToNativeSwapBridgePotPalletId;
    type Currency = EvmBalances;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = pallet_pot::DepositUnbalancedFungible<Self, PotInstanceTreasury>;
    type ExistentialDeposit = ConstU128<500>;
    type AccountStore = System;
    type MaxLocks = ConstU32<50>;
    // <https://github.com/paritytech/substrate/pull/12951>.
    type HoldIdentifier = ();
    type FreezeIdentifier = ();
    type MaxReserves = ();
    type MaxHolds = ConstU32<0>;
    type MaxFreezes = ConstU32<0>;
    type WeightInfo = weights::pallet_balances::WeightInfo<Runtime>;
}

parameter_types! {
    pub FeeMultiplier: pallet_transaction_payment::Multiplier = pallet_transaction_payment::Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<
        Balances,
        pallet_pot::DepositUnbalancedCurrency<Self, PotInstanceFees>,
    >;
    type OperationalFeeMultiplier = ConstU8<5>;
    type WeightToFee = frame_support::weights::ConstantMultiplier<
        Balance,
        ConstU128<{ constants::fees::WEIGHT_TO_FEE }>,
    >;
    type LengthToFee = frame_support::weights::ConstantMultiplier<
        Balance,
        ConstU128<{ constants::fees::LENGTH_TO_FEE }>,
    >;
    type FeeMultiplierUpdate = pallet_transaction_payment::ConstFeeMultiplier<FeeMultiplier>;
}

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
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
        #[allow(clippy::needless_borrow)]
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

impl pallet_bioauth::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RobonodePublicKey = robonode::PublicKey;
    type RobonodeSignature = Vec<u8>;
    type ValidatorPublicKey = BioauthId;
    type OpaqueAuthTicket = primitives_auth_ticket::OpaqueAuthTicket;
    type AuthTicketConverter = PrimitiveAuthTicketConverter;
    type ValidatorSetUpdater = ();
    type Moment = UnixMilliseconds;
    type DisplayMoment = display_moment::DisplayMoment;
    type CurrentMoment = CurrentMoment;
    type AuthenticationsExpireAfter = ConstU64<AUTHENTICATIONS_EXPIRE_AFTER>;
    type WeightInfo = weights::pallet_bioauth::WeightInfo<Runtime>;
    type MaxAuthentications = ConstU32<MAX_AUTHENTICATIONS>;
    type MaxNonces = ConstU32<MAX_NONCES>;
    type BeforeAuthHook = ();
    type AfterAuthHook = ();
    type DeauthenticationReason = DeauthenticationReason;
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

impl pallet_humanode_session::Config for Runtime {
    type ValidatorPublicKeyOf = IdentityValidatorIdOf;
    type BootnodeIdOf = sp_runtime::traits::Identity;
    type MaxBootnodeValidators = <Runtime as pallet_bootnodes::Config>::MaxBootnodes;
    type MaxBioauthValidators = <Runtime as pallet_bioauth::Config>::MaxAuthentications;
}

pub struct OffenceSlasher;

impl
    sp_staking::offence::OnOffenceHandler<
        AccountId,
        <Runtime as pallet_offences::Config>::IdentificationTuple,
        Weight,
    > for OffenceSlasher
{
    fn on_offence(
        offenders: &[sp_staking::offence::OffenceDetails<
            AccountId,
            <Runtime as pallet_offences::Config>::IdentificationTuple,
        >],
        _slash_fraction: &[Perbill],
        _session: sp_staking::SessionIndex,
        disable_strategy: sp_staking::offence::DisableStrategy,
    ) -> Weight {
        if disable_strategy == sp_staking::offence::DisableStrategy::Never {
            return Weight::zero();
        }
        let mut weight: Weight = Weight::zero();
        let weights = <Runtime as frame_system::Config>::DbWeight::get();
        let mut should_be_deauthenticated = Vec::with_capacity(offenders.len());
        for details in offenders {
            let (_offender, identity) = &details.offender;
            match identity {
                pallet_humanode_session::Identification::Bioauth(authentication) => {
                    should_be_deauthenticated.push(authentication.public_key.clone());
                }
                pallet_humanode_session::Identification::Bootnode(..) => {
                    // Never slash the bootnodes.
                }
            }
        }
        if !should_be_deauthenticated.is_empty() {
            let deauthenticated_public_keys =
                Bioauth::deauthenticate(should_be_deauthenticated, DeauthenticationReason::Offence);
            weight = weight.saturating_add(
                weights.reads_writes(
                    1,
                    deauthenticated_public_keys
                        .len()
                        .try_into()
                        .expect("casting usize to u64 never fails in 64bit and 32bit word cpus"),
                ),
            );
        }
        weight
    }
}

impl pallet_im_online::Config for Runtime {
    type AuthorityId = ImOnlineId;
    type RuntimeEvent = RuntimeEvent;
    type NextSessionRotation = Babe;
    type ValidatorSet = Historical;
    type ReportUnresponsiveness = Offences;
    type UnsignedPriority = ConstU64<{ TransactionPriority::MAX }>;
    type WeightInfo = weights::pallet_im_online::WeightInfo<Runtime>;
    type MaxKeys = ConstU32<MAX_KEYS>;
    type MaxPeerInHeartbeats = ConstU32<MAX_PEER_IN_HEARTBEATS>;
    type MaxPeerDataEncodingSize = ConstU32<MAX_PEER_DATA_ENCODING_SIZE>;
}

impl pallet_offences::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
    type OnOffenceHandler = OffenceSlasher;
}

const WEIGHT_MILLISECS_PER_BLOCK: u64 = EXPECTED_BLOCK_WEIGHT.ref_time()
    / frame_support::weights::constants::WEIGHT_REF_TIME_PER_MILLIS;
// An assertion to ensure this value is what we expect it to be here.
const_assert!(WEIGHT_MILLISECS_PER_BLOCK == 2000u64);

parameter_types! {
    pub BlockGasLimit: U256 = U256::from(constants::evm_fees::BLOCK_GAS_LIMIT);
    pub GasLimitPovSizeRatio: u64 = constants::evm_fees::GAS_LIMIT_POV_SIZE_RATIO;
    pub PrecompilesValue: FrontierPrecompiles<Runtime> = FrontierPrecompiles::<_>::default();
    pub WeightPerGas: Weight = Weight::from_parts(fp_evm::weight_per_gas(
        constants::evm_fees::BLOCK_GAS_LIMIT,
        NORMAL_DISPATCH_RATIO,
        WEIGHT_MILLISECS_PER_BLOCK,
    ), 0);
}

impl pallet_evm_system::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = EvmAccountId;
    type Index = Index;
    type AccountData = pallet_evm_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
}

impl pallet_evm_balances::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = EvmAccountId;
    type Balance = Balance;
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = EvmSystem;
    type DustRemoval = currency_swap::TreasuryPotProxy;
}

impl pallet_currency_swap::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AccountIdTo = EvmAccountId;
    type CurrencySwap = currency_swap::NativeToEvmOneToOne;
    type WeightInfo = ();
}

/// A simple fixed fee per gas calculator.
pub struct EvmFeePerGas;

impl fp_evm::FeeCalculator for EvmFeePerGas {
    fn min_gas_price() -> (U256, Weight) {
        (constants::evm_fees::FEE_PER_GAS.into(), Weight::zero())
    }
}

impl pallet_evm::Config for Runtime {
    type AccountProvider = EvmSystem;
    type FeeCalculator = EvmFeePerGas;
    type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
    type WeightPerGas = WeightPerGas;
    type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
    type CallOrigin = pallet_evm::EnsureAddressNever<EvmAccountId>;
    type WithdrawOrigin = pallet_evm::EnsureAddressNever<EvmAccountId>;
    type AddressMapping = pallet_evm::IdentityAddressMapping;
    type Currency = EvmBalances;
    type RuntimeEvent = RuntimeEvent;
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    type PrecompilesType = FrontierPrecompiles<Self>;
    type PrecompilesValue = PrecompilesValue;
    type ChainId = EthereumChainId;
    type BlockGasLimit = BlockGasLimit;
    type OnChargeTransaction =
        fixed_supply::EvmTransactionCharger<EvmBalances, currency_swap::FeesPotProxy>;
    type OnCreate = ();
    type FindAuthor = find_author::FindAuthorTruncated<
        find_author::FindAuthorFromSession<find_author::FindAuthorBabe, BabeId>,
    >;
    type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
    type Timestamp = Timestamp;
    type WeightInfo = pallet_evm::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const PostBlockAndTxnHashes: EthereumPostLogContent = EthereumPostLogContent::BlockAndTxnHashes;
}

impl pallet_ethereum::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
    type PostLogContent = PostBlockAndTxnHashes;
    type ExtraDataLength = ConstU32<EXTRA_DATA_LENGTH>;
}

impl pallet_chain_properties::Config for Runtime {}

impl pallet_ethereum_chain_id::Config for Runtime {}

impl pallet_evm_accounts_mapping::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Verifier = eth_sig::AccountClaimVerifier;
    type WeightInfo = weights::pallet_evm_accounts_mapping::WeightInfo<Runtime>;
}

parameter_types! {
    pub TokenClaimsPotAccountId: AccountId = TokenClaimsPot::account_id();
}

impl pallet_token_claims::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type PotAccountId = TokenClaimsPotAccountId;
    type VestingSchedule = <Self as pallet_vesting::Config>::Schedule;
    type VestingInterface = vesting::TokenClaimsInterface;
    type EthereumSignatureVerifier = eth_sig::TokenClaimVerifier;
    type WeightInfo = weights::pallet_token_claims::WeightInfo<Runtime>;
}

parameter_types! {
    pub VestingLockId: LockIdentifier = *b"hmnd/vs1";
}

impl pallet_vesting::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type LockId = VestingLockId;
    type Schedule = vesting::Schedule;
    type SchedulingDriver = vesting::SchedulingDriver;
    type WeightInfo = weights::pallet_vesting::WeightInfo<Runtime>;
}

impl pallet_multisig::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type DepositBase = ConstU128<1>;
    type DepositFactor = ConstU128<1>;
    type MaxSignatories = ConstU32<128>;
    type WeightInfo = weights::pallet_multisig::WeightInfo<Runtime>;
}

impl pallet_utility::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = weights::pallet_utility::WeightInfo<Runtime>;
}

parameter_types! {
    pub NativeToEvmSwapBridgePotAccountId: AccountId = NativeToEvmSwapBridgePot::account_id();
    pub EvmToNativeSwapBridgePotAccountId: EvmAccountId = EvmToNativeSwapBridgePot::account_id();
}

parameter_types! {
    pub TreasuryPotAccountId: AccountId = TreasuryPot::account_id();
}

impl pallet_balanced_currency_swap_bridges_initializer::Config for Runtime {
    type EvmAccountId = EvmAccountId;
    type NativeCurrency = Balances;
    type EvmCurrency = EvmBalances;
    type BalanceConverterEvmToNative = Identity;
    type BalanceConverterNativeToEvm = Identity;
    type NativeEvmBridgePot = NativeToEvmSwapBridgePotAccountId;
    type NativeTreasuryPot = TreasuryPotAccountId;
    type EvmNativeBridgePot = EvmToNativeSwapBridgePotAccountId;
    type ForceRebalanceAskCounter = ConstU16<0>;
    type WeightInfo = ();
}

pub struct EvmBalancesErc20Metadata;

impl pallet_erc20_support::Metadata for EvmBalancesErc20Metadata {
    fn name() -> &'static str {
        "Wrapped eHMND"
    }

    fn symbol() -> &'static str {
        "WeHMND"
    }

    fn decimals() -> u8 {
        18
    }
}

impl pallet_erc20_support::Config for Runtime {
    type AccountId = EvmAccountId;
    type Currency = EvmBalances;
    type Allowance = U256;
    type Metadata = EvmBalancesErc20Metadata;
}

frame_support::parameter_types! {
    pub PrecompilesAddresses: Vec<H160> =
        vec![
            frontier_precompiles::hash(precompiles_constants::BIOAUTH),
            frontier_precompiles::hash(precompiles_constants::EVM_ACCOUNTS_MAPPING),
            frontier_precompiles::hash(precompiles_constants::NATIVE_CURRENCY),
            frontier_precompiles::hash(precompiles_constants::CURRENCY_SWAP),
        ];
}

impl pallet_dummy_precompiles_code::Config for Runtime {
    type PrecompilesAddresses = PrecompilesAddresses;
    type ForceExecuteAskCounter = ConstU16<0>;
}

// Create the runtime by composing the FRAME pallets that were previously
// configured.
construct_runtime!(
    pub struct Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system = 0,
        Timestamp: pallet_timestamp = 2,
        ChainStartMoment: pallet_chain_start_moment = 3,
        Bootnodes: pallet_bootnodes = 4,
        Bioauth: pallet_bioauth = 5,
        // Must be before session.
        Babe: pallet_babe = 6,
        // Authorship must be before session.
        Authorship: pallet_authorship = 7,
        Balances: pallet_balances = 8,
        TreasuryPot: pallet_pot::<Instance1> = 9,
        FeesPot: pallet_pot::<Instance2> = 10,
        TokenClaimsPot: pallet_pot::<Instance3> = 11,
        TransactionPayment: pallet_transaction_payment = 12,
        Session: pallet_session = 13,
        Offences: pallet_offences = 14,
        Historical: pallet_session_historical = 15,
        HumanodeSession: pallet_humanode_session = 16,
        ChainProperties: pallet_chain_properties = 17,
        EthereumChainId: pallet_ethereum_chain_id = 18,
        Sudo: pallet_sudo = 19,
        Grandpa: pallet_grandpa = 20,
        Ethereum: pallet_ethereum = 21,
        EVM: pallet_evm = 22,
        ImOnline: pallet_im_online = 25,
        EvmAccountsMapping: pallet_evm_accounts_mapping = 26,
        TokenClaims: pallet_token_claims = 27,
        Vesting: pallet_vesting = 28,
        Multisig: pallet_multisig = 29,
        Utility: pallet_utility = 30,
        EvmSystem: pallet_evm_system = 31,
        EvmBalances: pallet_evm_balances = 32,
        NativeToEvmSwapBridgePot: pallet_pot::<Instance4> = 33,
        EvmToNativeSwapBridgePot: pallet_pot::<Instance5> = 34,
        CurrencySwap: pallet_currency_swap = 35,
        BalancedCurrencySwapBridgesInitializer: pallet_balanced_currency_swap_bridges_initializer = 36,
        EvmBalancesErc20Support: pallet_erc20_support = 37,
        DummyPrecompilesCode: pallet_dummy_precompiles_code = 38,
    }
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// The generic version of the `Extra` component of the [`UncheckedExtrinsic`] and
/// the [`SignedPayload`] of the current runtime implementation, but abstract around the runtime
/// config. Used internally to ensure we implement utilities in the generic fashion.
/// See [`SignedExtra`].
type GenericSignedExtra<R> = (
    frame_system::CheckSpecVersion<R>,
    frame_system::CheckTxVersion<R>,
    frame_system::CheckGenesis<R>,
    frame_system::CheckEra<R>,
    frame_system::CheckNonce<R>,
    frame_system::CheckWeight<R>,
    pallet_bioauth::CheckBioauthTx<R>,
    pallet_transaction_payment::ChargeTransactionPayment<R>,
    pallet_token_claims::CheckTokenClaim<R>,
);
/// The `Extra` component of the [`UncheckedExtrinsic`] and the [`SignedPayload`].
/// Effectively, additional data carried besides the call within the signed transactions.
pub type SignedExtra = GenericSignedExtra<Runtime>;
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    fp_self_contained::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

impl frame_system::offchain::CreateSignedTransaction<RuntimeCall> for Runtime {
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Self::RuntimeCall,
        public: <Self::Signature as sp_runtime::traits::Verify>::Signer,
        account: Self::AccountId,
        nonce: Self::Index,
    ) -> Option<(
        Self::RuntimeCall,
        <Self::Extrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
    )> {
        let tip = 0;
        let era = utils::current_era::<Self>();
        let extra = utils::create_extra::<Self>(nonce, era, tip);
        let raw_payload = SignedPayload::new(call, extra).ok()?;
        let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
        let address = Self::Lookup::unlookup(account);
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
    RuntimeCall: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = RuntimeCall;
}

impl fp_self_contained::SelfContainedCall for RuntimeCall {
    type SignedInfo = H160;

    fn is_self_contained(&self) -> bool {
        match self {
            RuntimeCall::Ethereum(call) => call.is_self_contained(),
            _ => false,
        }
    }

    fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
        match self {
            RuntimeCall::Ethereum(call) => call.check_self_contained(),
            _ => None,
        }
    }

    fn validate_self_contained(
        &self,
        info: &Self::SignedInfo,
        dispatch_info: &DispatchInfoOf<RuntimeCall>,
        len: usize,
    ) -> Option<TransactionValidity> {
        match self {
            RuntimeCall::Ethereum(call) => call.validate_self_contained(info, dispatch_info, len),
            _ => None,
        }
    }

    fn pre_dispatch_self_contained(
        &self,
        info: &Self::SignedInfo,
        dispatch_info: &DispatchInfoOf<RuntimeCall>,
        len: usize,
    ) -> Option<Result<(), TransactionValidityError>> {
        match self {
            RuntimeCall::Ethereum(call) => {
                call.pre_dispatch_self_contained(info, dispatch_info, len)
            }
            _ => None,
        }
    }

    fn apply_self_contained(
        self,
        info: Self::SignedInfo,
    ) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
        match self {
            call @ RuntimeCall::Ethereum(pallet_ethereum::Call::transact { .. }) => {
                Some(call.dispatch(RuntimeOrigin::from(
                    pallet_ethereum::RawOrigin::EthereumTransaction(info),
                )))
            }
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

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    frame_benchmarking::define_benchmarks!(
        [frame_benchmarking, BaselineBench::<Runtime>]
        [frame_system, SystemBench::<Runtime>]
        [pallet_babe, Babe]
        [pallet_balances, Balances]
        [pallet_bioauth, Bioauth]
        [pallet_evm_accounts_mapping, EvmAccountsMapping]
        [pallet_grandpa, Grandpa]
        [pallet_im_online, ImOnline]
        [pallet_multisig, Multisig]
        [pallet_timestamp, Timestamp]
        [pallet_token_claims, TokenClaims]
        [pallet_utility, Utility]
        [pallet_vesting, Vesting]
    );
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

        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }

        fn metadata_versions() -> sp_std::vec::Vec<u32> {
            Runtime::metadata_versions()
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
                AccountId::new(
                    <<KeystoreBioauthAccountId as sp_application_crypto::AppCrypto>::Public as sp_application_crypto::AppPublic>::Generic::from(id.clone()).0
                );
            let public_id = <KeystoreBioauthAccountId as frame_system::offchain::AppCrypto<
                    <Runtime as frame_system::offchain::SigningTypes>::Public,
                    <Runtime as frame_system::offchain::SigningTypes>::Signature
                >>::GenericPublic::from(id.clone());

            let keys = <Runtime as pallet_session::Config>::Keys::decode(&mut session_keys.as_slice())
                .map_err(|err| author_ext_api::CreateSignedSetKeysExtrinsicError::SessionKeysDecoding(err.to_string()))?;
            let session_call = pallet_session::Call::set_keys::<Runtime> { keys, proof: vec![] };
            let (call, (address, signature, extra)) =
                <Runtime as frame_system::offchain::CreateSignedTransaction<RuntimeCall>>::create_transaction::<KeystoreBioauthAccountId>(
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
                AccountId::new(
                    <<KeystoreBioauthAccountId as sp_application_crypto::AppCrypto>::Public as sp_application_crypto::AppPublic>::Generic::from(id.clone()).0
                );
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
        fn configuration() -> sp_consensus_babe::BabeConfiguration {
            let epoch_config = Babe::epoch_config().unwrap_or(BABE_GENESIS_EPOCH_CONFIG);
            // The choice of `c` parameter (where `1 - c` represents the
            // probability of a slot being empty), is done in accordance to the
            // slot duration and expected target block time, for safely
            // resisting network delays of maximum two seconds.
            // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
            sp_consensus_babe::BabeConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: <Self as pallet_babe::Config>::EpochDuration::get(),
                c: epoch_config.c,
                authorities: Babe::authorities().to_vec(),
                randomness: Babe::randomness(),
                allowed_slots: epoch_config.allowed_slots,
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
            pallet_evm::AccountCodes::<Runtime>::get(address)
        }

        fn author() -> H160 {
            <pallet_evm::Pallet<Runtime>>::find_author()
        }

        fn storage_at(address: H160, index: U256) -> H256 {
            let mut tmp = [0u8; 32];
            index.to_big_endian(&mut tmp);
            pallet_evm::AccountStorages::<Runtime>::get(address, H256::from_slice(&tmp[..]))
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
            let validate = true;
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
                validate,
                // TODO(#864): set proper values.
                None,
                None,
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
            let validate = true;
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
                validate,
                // TODO(#864): set proper values.
                None,
                None,
                &config,
            ).map_err(|err| err.error.into())
        }

        fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
            pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
        }

        fn current_block() -> Option<pallet_ethereum::Block> {
            pallet_ethereum::CurrentBlock::<Runtime>::get()
        }

        fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
            pallet_ethereum::CurrentReceipts::<Runtime>::get()
        }

        fn current_all() -> (
            Option<pallet_ethereum::Block>,
            Option<Vec<pallet_ethereum::Receipt>>,
            Option<Vec<TransactionStatus>>
        ) {
            (
                pallet_ethereum::CurrentBlock::<Runtime>::get(),
                pallet_ethereum::CurrentReceipts::<Runtime>::get(),
                pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
            )
        }

        fn extrinsic_filter(
            xts: Vec<<Block as BlockT>::Extrinsic>,
        ) -> Vec<EthereumTransaction> {
            xts.into_iter().filter_map(|xt| match xt.0.function {
                RuntimeCall::Ethereum(transact { transaction }) => Some(transaction),
                _ => None
            }).collect::<Vec<EthereumTransaction>>()
        }

        fn elasticity() -> Option<Permill> {
            None
        }

        fn gas_limit_multiplier_support() {}

        fn pending_block(
            xts: Vec<<Block as BlockT>::Extrinsic>,
        ) -> (Option<pallet_ethereum::Block>, Option<Vec<TransactionStatus>>) {
            for ext in xts {
                let _ = Executive::apply_extrinsic(ext);
            }

            Ethereum::on_finalize(
                // u32 (block number) is big enough for this overflow to be practically impossible.
                System::block_number().checked_add(1).unwrap()
            );

            (
                pallet_ethereum::CurrentBlock::<Runtime>::get(),
                pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
            )
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
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
        for Runtime
    {
        fn query_call_info(call: RuntimeCall, len: u32) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_call_info(call, len)
        }
        fn query_call_fee_details(call: RuntimeCall, len: u32) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_call_fee_details(call, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    impl pallet_vesting::api::VestingEvaluationApi<Block, AccountId, Balance> for Runtime {
        fn evaluate_lock(account: &AccountId) -> Result<Balance, pallet_vesting::api::EvaluationError> {
            Vesting::evaluate_lock(account)
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            let mut list = Vec::<BenchmarkList>::new();
            list_benchmarks!(list, extra);

            let storage_info = AllPalletsWithSystem::storage_info();

            (list, storage_info)
        }

        // Allow non local definitions lint for benchmark related code.
        #[allow(non_local_definitions)]
        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{Benchmarking, baseline, BenchmarkBatch, TrackedStorageKey};

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

            add_benchmarks!(params, batches);

            Ok(batches)
        }
    }

    #[cfg(feature = "try-runtime")]
    impl frame_try_runtime::TryRuntime<Block> for Runtime {
        fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here. If any of the pre/post migration checks fail, we shall stop
            // right here and right now.
            let weight = Executive::try_runtime_upgrade(checks).unwrap();
            (weight, BlockWeights::get().max_block)
        }

        fn execute_block(
            block: Block,
            state_root_check: bool,
            signature_check: bool,
            select: frame_try_runtime::TryStateSelect
        ) -> Weight {
            sp_tracing::info!(
                target: "humanode-runtime",
                "try-runtime: executing block {:?} / root checks: {:?} / signature check: {:?} / try-state-select: {:?}",
                block.header.hash(),
                state_root_check,
                signature_check,
                select,
            );
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here.
            Executive::try_execute_block(block, state_root_check, signature_check, select).unwrap()
        }
    }
}
