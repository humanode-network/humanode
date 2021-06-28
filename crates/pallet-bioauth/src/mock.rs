use crate as pallet_bioauth;
use codec::{Decode, Encode};
use frame_support::{parameter_types, traits::GenesisBuild};
use frame_system as system;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{crypto::Infallible, H256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Bioauth: pallet_bioauth::{Pallet, Call, Storage, Event<T>},
    }
);

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

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
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

impl pallet_bioauth::Config for Test {
    type Event = Event;
    type RobonodePublicKey = MockVerifier;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let config = pallet_bioauth::GenesisConfig::<Test>::default();
    config.assimilate_storage(&mut storage).unwrap();

    storage.into()
}
