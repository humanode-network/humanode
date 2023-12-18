use frame_support::{parameter_types, traits::ConstU64};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

use crate::{self as pallet_chain_properties};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

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
        ChainProperties: pallet_chain_properties::{Pallet, Storage, Config},
    }
);

parameter_types! {
    pub SS58Prefix: u16 = ChainProperties::ss58_prefix();
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
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_chain_properties::Config for Test {}

/// Build test externalities from the custom genesis.
/// Using this call requires manual assertions on the genesis init logic.
pub fn new_test_ext_with(
    config: pallet_chain_properties::GenesisConfig,
) -> sp_io::TestExternalities {
    // Build genesis.
    let config = GenesisConfig {
        chain_properties: config,
        ..Default::default()
    };
    let storage = config.build_storage().unwrap();

    // Make test externalities from the storage.
    storage.into()
}
