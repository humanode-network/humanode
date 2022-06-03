use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::parameter_types;
use frame_system as system;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{crypto::Infallible, H256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

use crate::{self as pallet_bioauth, weights, AuthTicket, TryConvert};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Benchmark>;
type Block = frame_system::mocking::MockBlock<Benchmark>;

/// A timestamp: milliseconds since the unix epoch.
pub type UnixMilliseconds = u64;

/// An index to a block.
pub type BlockNumber = u64;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Benchmark where
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

impl crate::Verifier<Vec<u8>> for MockVerifier {
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

impl crate::ValidatorSetUpdater<ValidatorPublicKey> for MockValidatorSetUpdater {
    fn update_validators_set<'a, I: Iterator<Item = &'a ValidatorPublicKey> + 'a>(
        _validator_public_keys: I,
    ) where
        ValidatorPublicKey: 'a,
    {
        ()
    }

    fn init_validators_set<'a, I: Iterator<Item = &'a ValidatorPublicKey> + 'a>(
        _validator_public_keys: I,
    ) where
        ValidatorPublicKey: 'a,
    {
        ()
    }
}

pub struct MockCurrentMomentProvider;

impl crate::CurrentMoment<UnixMilliseconds> for MockCurrentMomentProvider {
    fn now() -> UnixMilliseconds {
        UnixMilliseconds::default()
    }
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Benchmark {
    type BaseCallFilter = frame_support::traits::Everything;
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
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

const TIMESTAMP_SECOND: UnixMilliseconds = 1000;
const TIMESTAMP_MINUTE: UnixMilliseconds = 60 * TIMESTAMP_SECOND;

pub const AUTHENTICATIONS_EXPIRE_AFTER: UnixMilliseconds = TIMESTAMP_MINUTE;

parameter_types! {
    pub const AuthenticationsExpireAfter: UnixMilliseconds = AUTHENTICATIONS_EXPIRE_AFTER;
    pub const MaxAuthentications: u32 = 512;
    pub const MaxNonces: u32 = 512;
}

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

impl pallet_bioauth::Config for Benchmark {
    type Event = Event;
    type RobonodePublicKey = MockVerifier;
    type RobonodeSignature = Vec<u8>;
    type ValidatorPublicKey = ValidatorPublicKey;
    type OpaqueAuthTicket = MockOpaqueAuthTicket;
    type AuthTicketCoverter = MockAuthTicketConverter;
    type ValidatorSetUpdater = MockValidatorSetUpdater;
    type Moment = UnixMilliseconds;
    type DisplayMoment = DisplayMoment;
    type CurrentMoment = MockCurrentMomentProvider;
    type AuthenticationsExpireAfter = AuthenticationsExpireAfter;
    type WeightInfo = weights::SubstrateWeight<Benchmark>;
    type MaxAuthentications = MaxAuthentications;
    type MaxNonces = MaxNonces;
    type BeforeAuthHook = ();
    type AfterAuthHook = ();
}

#[cfg(feature = "runtime-benchmarks")]
impl crate::benchmarking::AuthTicketSigner<Benchmark> for Benchmark {
    fn sign(_ticket: &MockOpaqueAuthTicket) -> Vec<u8> {
        vec![0; 64]
    }
}

#[cfg(feature = "runtime-benchmarks")]
impl crate::benchmarking::AuthTicketBuilder<Benchmark> for Benchmark {
    fn build(public_key: Vec<u8>, nonce: Vec<u8>) -> MockOpaqueAuthTicket {
        let public_key_fixed_size: [u8; 32] = public_key.try_into().unwrap();
        let opaque_auth_ticket = AuthTicket {
            public_key: public_key_fixed_size,
            nonce,
        };
        MockOpaqueAuthTicket(opaque_auth_ticket)
    }
}

/// Build benchmark externalities from the default genesis.
pub fn new_benchmark_ext() -> sp_io::TestExternalities {
    // Build externalities with default genesis.
    let externalities = new_benchmark_ext_with(Default::default());

    // Return ready-to-use externalities.
    externalities
}

/// Build benchmark externalities from the custom genesis.
/// Using this call requires manual assertions on the genesis init logic.
pub fn new_benchmark_ext_with(
    config: pallet_bioauth::GenesisConfig<Benchmark>,
) -> sp_io::TestExternalities {
    // Build genesis.
    let config = GenesisConfig {
        bioauth: config,
        ..Default::default()
    };
    let storage = config.build_storage().unwrap();

    // Make test externalities from the storage.
    storage.into()
}
