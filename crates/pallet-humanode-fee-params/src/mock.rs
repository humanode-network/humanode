//! The mock for the pallet.

use frame_support::{
    sp_io,
    sp_runtime::{
        testing::{Header, H256},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage as _,
    },
    traits::{ConstU32, ConstU64},
    weights::constants::RocksDbWeight,
};

use crate::{self as pallet_humanode_fee_params};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub struct Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        HumanodeFeeParams: pallet_humanode_fee_params::{Pallet, Call, Storage, Config, Event<T>},
    }
);

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = RocksDbWeight;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<u64>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl pallet_humanode_fee_params::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

/// Build the test externalities with the default genesis config of the pallet.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let genesis_config = GenesisConfig {
        system: Default::default(),
        humanode_fee_params: Default::default(),
    };
    new_test_ext_with(genesis_config)
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext_with(genesis_config: GenesisConfig) -> sp_io::TestExternalities {
    let storage = genesis_config.build_storage().unwrap();
    storage.into()
}

/// Build the test externalities where the pallet genesis has not been applied, as it is
/// on the chains that upgrade from a runtime where this pallet did not exist.
pub fn new_test_ext_without_pallet_genesis() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    storage.into()
}
