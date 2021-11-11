use std::cell::RefCell;

use crate::{self as pallet_bioauth, AuthTicket, TryConvert};
use codec::{Decode, Encode};
use frame_support::parameter_types;
use frame_system as system;
use mockall::{mock, predicate};
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
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Bioauth: pallet_bioauth::{Pallet, Config<T>, Call, Storage, Event<T>, ValidateUnsigned},
    }
);

#[derive(PartialEq, Eq, Default, Clone, Encode, Decode, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct MockOpaqueAuthTicket(pub AuthTicket<Vec<u8>>);

impl AsRef<[u8]> for MockOpaqueAuthTicket {
    fn as_ref(&self) -> &[u8] {
        panic!("should be unused in tests")
    }
}

pub struct MockAuthTicketConverter;

impl TryConvert<MockOpaqueAuthTicket, AuthTicket<Vec<u8>>> for MockAuthTicketConverter {
    type Error = Infallible;

    fn try_convert(value: MockOpaqueAuthTicket) -> Result<AuthTicket<Vec<u8>>, Self::Error> {
        Ok(value.0)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct MockVerifier;

impl super::Verifier<Vec<u8>> for MockVerifier {
    type Error = Infallible;

    fn verify<'a, D>(&self, _data: D, signature: Vec<u8>) -> Result<bool, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a,
    {
        Ok(signature.starts_with(b"should_be_valid"))
    }
}

mock! {
    pub ValidatorSetUpdater {
        pub fn update_validators_set(&self, validator_public_keys: Vec<Vec<u8>>);
        pub fn init_validators_set(&self, validator_public_keys: Vec<Vec<u8>>);
    }
}

thread_local! {
    pub static MOCK_VALIDATOR_SET_UPDATER: RefCell<MockValidatorSetUpdater> = RefCell::new(MockValidatorSetUpdater::new());
}

impl super::ValidatorSetUpdater<Vec<u8>> for MockValidatorSetUpdater {
    fn update_validators_set<'a, I: Iterator<Item = &'a Vec<u8>> + 'a>(validator_public_keys: I)
    where
        Vec<u8>: 'a,
    {
        MOCK_VALIDATOR_SET_UPDATER.with(|val| {
            val.borrow_mut()
                .update_validators_set(validator_public_keys.cloned().collect())
        });
    }

    fn init_validators_set<'a, I: Iterator<Item = &'a Vec<u8>> + 'a>(validator_public_keys: I)
    where
        Vec<u8>: 'a,
    {
        MOCK_VALIDATOR_SET_UPDATER.with(|val| {
            val.borrow_mut()
                .init_validators_set(validator_public_keys.cloned().collect())
        });
    }
}

pub fn with_mock_validator_set_updater<F, R>(f: F) -> R
where
    F: FnOnce(&mut MockValidatorSetUpdater) -> R,
{
    MOCK_VALIDATOR_SET_UPDATER.with(|var| f(&mut *var.borrow_mut()))
}

mock! {
    pub CurrentMomentProvider {
        pub fn now(&self) -> UnixMilliseconds;
    }
}

thread_local! {
    pub static MOCK_CURRENT_MOMENT_PROVIDER: RefCell<MockCurrentMomentProvider> = RefCell::new(MockCurrentMomentProvider::new());
}

impl super::CurrentMoment<UnixMilliseconds> for MockCurrentMomentProvider {
    fn now() -> UnixMilliseconds {
        MOCK_CURRENT_MOMENT_PROVIDER.with(|val| val.borrow().now())
    }
}

pub fn with_mock_current_moment_provider<F, R>(f: F) -> R
where
    F: FnOnce(&mut MockCurrentMomentProvider) -> R,
{
    MOCK_CURRENT_MOMENT_PROVIDER.with(|var| f(&mut *var.borrow_mut()))
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::AllowAll;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
}

pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

const TIMESTAMP_SECOND: UnixMilliseconds = 1000;
const TIMESTAMP_MINUTE: UnixMilliseconds = 60 * TIMESTAMP_SECOND;

pub const AUTHENTICATIONS_EXPIRE_AFTER: UnixMilliseconds = TIMESTAMP_MINUTE;

parameter_types! {
    pub const AuthenticationsExpireAfter: UnixMilliseconds = AUTHENTICATIONS_EXPIRE_AFTER;
}

impl pallet_bioauth::Config for Test {
    type Event = Event;
    type RobonodePublicKey = MockVerifier;
    type RobonodeSignature = Vec<u8>;
    type ValidatorPublicKey = Vec<u8>;
    type OpaqueAuthTicket = MockOpaqueAuthTicket;
    type AuthTicketCoverter = MockAuthTicketConverter;
    type ValidatorSetUpdater = MockValidatorSetUpdater;
    type Moment = UnixMilliseconds;
    type CurrentMoment = MockCurrentMomentProvider;
    type AuthenticationsExpireAfter = AuthenticationsExpireAfter;
}

/// Build test externalities from the default genesis.
pub fn new_test_ext() -> sp_io::TestExternalities {
    // Add mock validator set updater expectation for the genesis validators set init.
    with_mock_validator_set_updater(|mock| {
        mock.expect_init_validators_set()
            .with(predicate::eq(vec![]))
            .return_const(());
    });

    // Build externalities with default genesis.
    let externalities = new_test_ext_with(Default::default());

    // Assert the genesis validators set init.
    with_mock_validator_set_updater(|mock| {
        mock.checkpoint();
    });

    // Return ready-to-use externalities.
    externalities
}

/// Build test externalities from the custom genesis.
/// Using this call requires manual assertions on the genesis init logic.
pub fn new_test_ext_with(config: pallet_bioauth::GenesisConfig<Test>) -> sp_io::TestExternalities {
    // Build genesis.
    let config = GenesisConfig {
        bioauth: config,
        ..Default::default()
    };
    let storage = config.build_storage().unwrap();

    // Make test externalities from the storage.
    storage.into()
}
