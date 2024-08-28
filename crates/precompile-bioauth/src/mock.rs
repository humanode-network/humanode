// Allow simple integer arithmetic in tests.
#![allow(clippy::arithmetic_side_effects)]

use codec::{Decode, Encode, MaxEncodedLen};
use fp_evm::{Context, ExitError, ExitReason, PrecompileHandle, Transfer};
use frame_support::traits::{ConstU16, ConstU32, ConstU64};
use frame_system as system;
use mockall::predicate::*;
use mockall::*;
use pallet_bioauth::{AuthTicket, TryConvert};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{crypto::Infallible, H256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

/// A timestamp: milliseconds since the unix epoch.
pub type UnixMilliseconds = u64;

/// An index to a block.
pub type BlockNumber = u64;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub struct Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Bioauth: pallet_bioauth::{Pallet, Config<T>, Call, Storage, Event<T>, ValidateUnsigned},
    }
);

/// Validator public key. Should be bounded.
pub type ValidatorPublicKey = [u8; 32];

#[derive(PartialEq, Eq, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct MockOpaqueAuthTicket(pub AuthTicket<ValidatorPublicKey>);

impl AsRef<[u8]> for MockOpaqueAuthTicket {
    fn as_ref(&self) -> &[u8] {
        panic!("should be unused in tests")
    }
}

impl From<Vec<u8>> for MockOpaqueAuthTicket {
    fn from(bytes: Vec<u8>) -> Self {
        Self::decode(&mut bytes.as_ref()).unwrap()
    }
}

pub struct MockAuthTicketConverter;

impl TryConvert<MockOpaqueAuthTicket, AuthTicket<ValidatorPublicKey>> for MockAuthTicketConverter {
    type Error = Infallible;

    fn try_convert(
        value: MockOpaqueAuthTicket,
    ) -> Result<AuthTicket<ValidatorPublicKey>, Self::Error> {
        Ok(value.0)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct MockVerifier;

impl pallet_bioauth::Verifier<Vec<u8>> for MockVerifier {
    type Error = Infallible;

    fn verify<'a, D>(&self, _data: D, _signature: Vec<u8>) -> Result<bool, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a,
    {
        Ok(true)
    }
}

impl MaxEncodedLen for MockVerifier {
    fn max_encoded_len() -> usize {
        panic!("should be unused in tests")
    }
}

pub struct MockValidatorSetUpdater;

impl pallet_bioauth::ValidatorSetUpdater<ValidatorPublicKey> for MockValidatorSetUpdater {
    fn update_validators_set<'a, I: Iterator<Item = &'a ValidatorPublicKey> + 'a>(
        _validator_public_keys: I,
    ) where
        ValidatorPublicKey: 'a,
    {
    }

    fn init_validators_set<'a, I: Iterator<Item = &'a ValidatorPublicKey> + 'a>(
        _validator_public_keys: I,
    ) where
        ValidatorPublicKey: 'a,
    {
    }
}

pub struct MockCurrentMomentProvider;

impl pallet_bioauth::CurrentMoment<UnixMilliseconds> for MockCurrentMomentProvider {
    fn now() -> UnixMilliseconds {
        UnixMilliseconds::default()
    }
}

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

const TIMESTAMP_SECOND: UnixMilliseconds = 1000;
const TIMESTAMP_MINUTE: UnixMilliseconds = 60 * TIMESTAMP_SECOND;

pub const AUTHENTICATIONS_EXPIRE_AFTER: UnixMilliseconds = TIMESTAMP_MINUTE;
pub const MAX_AUTHENTICATIONS: u32 = 512;
pub const MAX_NONCES: u32 = 512;
pub const MAX_BLACK_LISTED_VALIDATORS_PUBLIC_KEYS: u32 = 512;

pub struct DisplayMoment;

impl From<UnixMilliseconds> for DisplayMoment {
    fn from(_moment: UnixMilliseconds) -> Self {
        panic!("should be unused in tests")
    }
}

impl core::fmt::Display for DisplayMoment {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        panic!("should be unused in tests")
    }
}

impl pallet_bioauth::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type RobonodePublicKey = MockVerifier;
    type RobonodeSignature = Vec<u8>;
    type ValidatorPublicKey = ValidatorPublicKey;
    type OpaqueAuthTicket = MockOpaqueAuthTicket;
    type AuthTicketConverter = MockAuthTicketConverter;
    type ValidatorSetUpdater = MockValidatorSetUpdater;
    type Moment = UnixMilliseconds;
    type DisplayMoment = DisplayMoment;
    type CurrentMoment = MockCurrentMomentProvider;
    type AuthenticationsExpireAfter = ConstU64<AUTHENTICATIONS_EXPIRE_AFTER>;
    type WeightInfo = ();
    type MaxAuthentications = ConstU32<MAX_AUTHENTICATIONS>;
    type MaxNonces = ConstU32<MAX_NONCES>;
    type MaxBlackListedValidatorPublicKeys = ConstU32<MAX_BLACK_LISTED_VALIDATORS_PUBLIC_KEYS>;
    type BeforeAuthHook = ();
    type AfterAuthHook = ();
    type DeauthenticationReason = ();
}

mock! {
    pub PrecompileHandle {}

    impl PrecompileHandle for PrecompileHandle {
        fn call(
            &mut self,
            to: sp_core::H160,
            transfer: Option<Transfer>,
            input: Vec<u8>,
            gas_limit: Option<u64>,
            is_static: bool,
            context: &Context,
        ) -> (ExitReason, Vec<u8>);

        fn record_cost(&mut self, cost: u64) -> Result<(), ExitError>;

        fn remaining_gas(&self) -> u64;

        fn log(&mut self, address: sp_core::H160, topics: Vec<sp_core::H256>, data: Vec<u8>) -> Result<(), ExitError>;

        fn code_address(&self) -> sp_core::H160;

        fn input(&self) -> &[u8];

        fn context(&self) -> &Context;

        fn is_static(&self) -> bool;

        fn gas_limit(&self) -> Option<u64>;

        fn record_external_cost(
            &mut self,
            ref_time: Option<u64>,
            proof_size: Option<u64>,
        ) -> Result<(), ExitError>;

        fn refund_external_cost(&mut self, ref_time: Option<u64>, proof_size: Option<u64>);
    }
}

/// Build test externalities from the default genesis.
pub fn new_test_ext() -> sp_io::TestExternalities {
    // Build genesis.
    let config = GenesisConfig {
        ..Default::default()
    };
    let storage = config.build_storage().unwrap();

    // Make test externalities from the storage.
    storage.into()
}
