use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    parameter_types, sp_io,
    sp_runtime::{
        self,
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        BoundedVec, BuildStorage,
    },
    traits::{ConstU16, ConstU64},
};
use frame_system as system;
use mockall::mock;
use pallet_session::historical as pallet_session_historical;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::{crypto::DeriveError, ConstU32, H256};

use crate::{self as pallet_humanode_offences};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub type BlockNumber = u64;
pub type AccountId = u64;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub struct Test
    where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Bootnodes: pallet_bootnodes,
        Bioauth: pallet_bioauth,
        Session: pallet_session,
        Historical: pallet_session_historical,
        HumanodeSession: pallet_humanode_session,
        HumanodeOffences: pallet_humanode_offences,
    }
);

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
    type AccountId = AccountId;
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

impl pallet_bootnodes::Config for Test {
    type BootnodeId = AccountId;
    type MaxBootnodes = ConstU32<3>;
}

#[derive(PartialEq, Eq, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct MockOpaqueAuthTicket;

impl AsRef<[u8]> for MockOpaqueAuthTicket {
    fn as_ref(&self) -> &[u8] {
        panic!("should be unused in tests")
    }
}

#[derive(
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Encode,
    Decode,
    Hash,
    Debug,
    TypeInfo,
    Serialize,
    Deserialize,
)]
pub struct MockVerifier;

impl pallet_bioauth::Verifier<Vec<u8>> for MockVerifier {
    type Error = DeriveError;

    fn verify<'a, D>(&self, _data: D, _signature: Vec<u8>) -> Result<bool, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a,
    {
        panic!("should be unused in tests")
    }
}

impl MaxEncodedLen for MockVerifier {
    fn max_encoded_len() -> usize {
        panic!("should be unused in tests")
    }
}

pub struct MockAuthTicketConverter;

impl pallet_bioauth::TryConvert<MockOpaqueAuthTicket, pallet_bioauth::AuthTicket<AccountId>>
    for MockAuthTicketConverter
{
    type Error = DeriveError;

    fn try_convert(
        _value: MockOpaqueAuthTicket,
    ) -> Result<pallet_bioauth::AuthTicket<AccountId>, Self::Error> {
        panic!("should be unused in tests")
    }
}

pub struct MockCurrentMoment;

impl pallet_bioauth::CurrentMoment<u64> for MockCurrentMoment {
    fn now() -> u64 {
        panic!("should be unused in tests")
    }
}

pub struct MockDisplayMoment;

impl From<u64> for MockDisplayMoment {
    fn from(_moment: u64) -> Self {
        panic!("should be unused in tests")
    }
}

impl core::fmt::Display for MockDisplayMoment {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        panic!("should be unused in tests")
    }
}

/// Define a possible deauthentication reason.
#[derive(Clone, PartialEq, Debug, Encode, Decode, TypeInfo)]
pub enum DeauthenticationReason {
    /// Some offence has been received.
    Offence,
}

impl pallet_bioauth::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type RobonodePublicKey = MockVerifier;
    type RobonodeSignature = Vec<u8>;
    type ValidatorPublicKey = AccountId;
    type OpaqueAuthTicket = MockOpaqueAuthTicket;
    type AuthTicketConverter = MockAuthTicketConverter;
    type ValidatorSetUpdater = ();
    type Moment = u64;
    type DisplayMoment = MockDisplayMoment;
    type CurrentMoment = MockCurrentMoment;
    type AuthenticationsExpireAfter = ConstU64<10>;
    type WeightInfo = ();
    type MaxAuthentications = ConstU32<5>;
    type MaxNonces = ConstU32<5>;
    type BeforeAuthHook = ();
    type AfterAuthHook = ();
    type DeauthenticationReason = DeauthenticationReason;
}

mock! {
    pub ShouldEndSession {
        pub fn should_end_session(now: BlockNumber) -> bool;
    }
}

impl pallet_session::ShouldEndSession<BlockNumber> for MockShouldEndSession {
    fn should_end_session(now: BlockNumber) -> bool {
        MockShouldEndSession::should_end_session(now)
    }
}

pub struct IdentityValidatorIdOf;
impl sp_runtime::traits::Convert<AccountId, Option<AccountId>> for IdentityValidatorIdOf {
    fn convert(account_id: AccountId) -> Option<AccountId> {
        Some(account_id)
    }
}

impl pallet_session::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type ValidatorId = AccountId;
    type ValidatorIdOf = IdentityValidatorIdOf;
    type ShouldEndSession = MockShouldEndSession;
    type NextSessionRotation = ();
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, HumanodeSession>;
    type SessionHandler = pallet_session::TestSessionHandler;
    type Keys = sp_runtime::testing::UintAuthorityId;
    type WeightInfo = ();
}

impl pallet_session::historical::Config for Test {
    type FullIdentification = pallet_humanode_session::IdentificationFor<Self>;
    type FullIdentificationOf = pallet_humanode_session::CurrentSessionIdentificationOf<Self>;
}

impl pallet_humanode_session::Config for Test {
    type ValidatorPublicKeyOf = IdentityValidatorIdOf;
    type BootnodeIdOf = sp_runtime::traits::Identity;
    type MaxBootnodeValidators = <Test as pallet_bootnodes::Config>::MaxBootnodes;
    type MaxBioauthValidators = <Test as pallet_bioauth::Config>::MaxAuthentications;
    type MaxBannedAccounts = <Test as pallet_bioauth::Config>::MaxAuthentications;
    type WeightInfo = ();
}

parameter_types! {
    pub const DeauthenticationReasonOnOffenceReport: DeauthenticationReason = DeauthenticationReason::Offence;
}

impl pallet_humanode_offences::Config for Test {
    type DeauthenticationReasonOnOffenceReport = DeauthenticationReasonOnOffenceReport;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let genesis_config = GenesisConfig {
        session: pallet_session::GenesisConfig {
            keys: vec![
                (42, 42, sp_runtime::testing::UintAuthorityId(42)),
                (43, 43, sp_runtime::testing::UintAuthorityId(43)),
                (44, 44, sp_runtime::testing::UintAuthorityId(44)),
                // Not bootnode.
                (1, 1, sp_runtime::testing::UintAuthorityId(1)),
            ],
        },
        bootnodes: pallet_bootnodes::GenesisConfig {
            bootnodes: BoundedVec::truncate_from(vec![42, 43, 44]),
        },
        bioauth: pallet_bioauth::GenesisConfig {
            active_authentications: BoundedVec::try_from(vec![pallet_bioauth::Authentication {
                public_key: 1,
                expires_at: 1000,
            }])
            .unwrap(),
            ..Default::default()
        },
        ..Default::default()
    };

    let storage = genesis_config.build_storage().unwrap();

    // Make test externalities from the storage.
    storage.into()
}

pub fn runtime_lock() -> std::sync::MutexGuard<'static, ()> {
    static MOCK_RUNTIME_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    // Ignore the poisoning for the tests that panic.
    // We only care about concurrency here, not about the poisoning.
    match MOCK_RUNTIME_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub trait TestExternalitiesExt {
    fn execute_with_ext<R, E>(&mut self, execute: E) -> R
    where
        E: for<'e> FnOnce(&'e ()) -> R;
}

impl TestExternalitiesExt for frame_support::sp_io::TestExternalities {
    fn execute_with_ext<R, E>(&mut self, execute: E) -> R
    where
        E: for<'e> FnOnce(&'e ()) -> R,
    {
        let guard = runtime_lock();
        let result = self.execute_with(|| execute(&guard));
        drop(guard);
        result
    }
}
