use std::cell::RefCell;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::{ConstU16, ConstU32, ConstU64};
use frame_system as system;
use mockall::{mock, predicate};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{crypto::Infallible, H256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

use crate::{self as pallet_bioauth, AuthTicket, TryConvert};

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
pub struct MockAuthTicketConverter;

impl TryConvert<MockOpaqueAuthTicket, AuthTicket<ValidatorPublicKey>> for MockAuthTicketConverter {
    type Error = Infallible;

    fn try_convert(
        value: MockOpaqueAuthTicket,
    ) -> Result<AuthTicket<ValidatorPublicKey>, Self::Error> {
        Ok(value.0)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum MockVerifier {
    A,
    B,
}

impl Default for MockVerifier {
    fn default() -> Self {
        Self::A
    }
}

impl crate::Verifier<Vec<u8>> for MockVerifier {
    type Error = Infallible;

    fn verify<'a, D>(&self, _data: D, signature: Vec<u8>) -> Result<bool, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a,
    {
        Ok(signature.starts_with(b"should_be_valid"))
    }
}

impl MaxEncodedLen for MockVerifier {
    fn max_encoded_len() -> usize {
        panic!("should be unused in tests")
    }
}

mock! {
    pub ValidatorSetUpdater {
        pub fn update_validators_set(&self, validator_public_keys: Vec<ValidatorPublicKey>);
        pub fn init_validators_set(&self, validator_public_keys: Vec<ValidatorPublicKey>);
    }
}

thread_local! {
    pub static MOCK_VALIDATOR_SET_UPDATER: RefCell<MockValidatorSetUpdater> = RefCell::new(MockValidatorSetUpdater::new());
}

impl crate::ValidatorSetUpdater<ValidatorPublicKey> for MockValidatorSetUpdater {
    fn update_validators_set<'a, I: Iterator<Item = &'a ValidatorPublicKey> + 'a>(
        validator_public_keys: I,
    ) where
        ValidatorPublicKey: 'a,
    {
        MOCK_VALIDATOR_SET_UPDATER.with(|val| {
            val.borrow_mut()
                .update_validators_set(validator_public_keys.copied().collect())
        });
    }

    fn init_validators_set<'a, I: Iterator<Item = &'a ValidatorPublicKey> + 'a>(
        validator_public_keys: I,
    ) where
        ValidatorPublicKey: 'a,
    {
        MOCK_VALIDATOR_SET_UPDATER.with(|val| {
            val.borrow_mut()
                .init_validators_set(validator_public_keys.copied().collect())
        });
    }
}

pub fn with_mock_validator_set_updater<F, R>(f: F) -> R
where
    F: FnOnce(&mut MockValidatorSetUpdater) -> R,
{
    MOCK_VALIDATOR_SET_UPDATER.with(|var| f(&mut var.borrow_mut()))
}

mock! {
    pub CurrentMomentProvider {
        pub fn now(&self) -> UnixMilliseconds;
    }
}

thread_local! {
    pub static MOCK_CURRENT_MOMENT_PROVIDER: RefCell<MockCurrentMomentProvider> = RefCell::new(MockCurrentMomentProvider::new());
}

impl crate::CurrentMoment<UnixMilliseconds> for MockCurrentMomentProvider {
    fn now() -> UnixMilliseconds {
        MOCK_CURRENT_MOMENT_PROVIDER.with(|val| val.borrow().now())
    }
}

pub fn with_mock_current_moment_provider<F, R>(f: F) -> R
where
    F: FnOnce(&mut MockCurrentMomentProvider) -> R,
{
    MOCK_CURRENT_MOMENT_PROVIDER.with(|var| f(&mut var.borrow_mut()))
}

mock! {
    pub BeforeAuthHookProvider {
        pub fn hook(&self, authentication: &crate::Authentication<ValidatorPublicKey, UnixMilliseconds>) -> Result<(), sp_runtime::DispatchError>;
    }
}
mock! {
    pub AfterAuthHookProvider {
        pub fn hook(&self, before_hook_data: ());
    }
}

thread_local! {
    pub static MOCK_BEFORE_AUTH_HOOK_PROVIDER: RefCell<MockBeforeAuthHookProvider> = RefCell::new(MockBeforeAuthHookProvider::new());
    pub static MOCK_AFTER_AUTH_HOOK_PROVIDER: RefCell<MockAfterAuthHookProvider> = RefCell::new(MockAfterAuthHookProvider::new());
}

impl crate::BeforeAuthHook<ValidatorPublicKey, UnixMilliseconds> for MockBeforeAuthHookProvider {
    type Data = ();

    fn hook(
        authentication: &crate::Authentication<ValidatorPublicKey, UnixMilliseconds>,
    ) -> Result<Self::Data, sp_runtime::DispatchError> {
        MOCK_BEFORE_AUTH_HOOK_PROVIDER.with(|val| val.borrow().hook(authentication))
    }
}
impl crate::AfterAuthHook<()> for MockAfterAuthHookProvider {
    fn hook(before_hook_data: ()) {
        MOCK_AFTER_AUTH_HOOK_PROVIDER.with(|val| val.borrow().hook(before_hook_data))
    }
}

pub fn with_mock_before_auth_hook_provider<F, R>(f: F) -> R
where
    F: FnOnce(&mut MockBeforeAuthHookProvider) -> R,
{
    MOCK_BEFORE_AUTH_HOOK_PROVIDER.with(|var| f(&mut var.borrow_mut()))
}

pub fn with_mock_after_auth_hook_provider<F, R>(f: F) -> R
where
    F: FnOnce(&mut MockAfterAuthHookProvider) -> R,
{
    MOCK_AFTER_AUTH_HOOK_PROVIDER.with(|var| f(&mut var.borrow_mut()))
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

pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

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
    type BeforeAuthHook = MockBeforeAuthHookProvider;
    type AfterAuthHook = MockAfterAuthHookProvider;
    type DeauthenticationReason = ();
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
