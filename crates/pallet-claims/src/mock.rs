//! The mock for the claims pallet.

use frame_support::{
    ord_parameter_types, parameter_types,
    traits::{ConstU32, ConstU64, GenesisBuild},
};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, Identity, IdentityLookup},
};

use crate::{self as pallet_claims, secp_utils::*, weights::TestWeightInfo};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Vesting: pallet_vesting::{Pallet, Call, Storage, Config<T>, Event<T>},
        Claims: pallet_claims::{Pallet, Call, Storage, Config<T>, Event<T>, ValidateUnsigned},
    }
);

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
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
    type Lookup = IdentityLookup<u64>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
    type Balance = u64;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<1>;
    type AccountStore = System;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl pallet_vesting::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type BlockNumberToBalance = Identity;
    type MinVestedTransfer = ConstU64<1>;
    type WeightInfo = ();
    const MAX_VESTING_SCHEDULES: u32 = 28;
}

parameter_types! {
    pub Prefix: &'static [u8] = b"Pay RUSTs to the TEST account:";
}
ord_parameter_types! {
    pub const Six: u64 = 6;
}

impl pallet_claims::Config for Test {
    type Event = Event;
    type VestingSchedule = Vesting;
    type Prefix = Prefix;
    type MoveClaimOrigin = frame_system::EnsureSignedBy<Six, u64>;
    type WeightInfo = TestWeightInfo;
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    // We use default for brevity, but you can configure as desired if needed.
    pallet_balances::GenesisConfig::<Test>::default()
        .assimilate_storage(&mut t)
        .unwrap();
    pallet_claims::GenesisConfig::<Test> {
        claims: vec![
            (eth(&alice()), 100, None, None),
            (
                eth(&dave()),
                200,
                None,
                Some(pallet_claims::StatementKind::Regular),
            ),
            (
                eth(&eve()),
                300,
                Some(42),
                Some(pallet_claims::StatementKind::Saft),
            ),
            (eth(&frank()), 400, Some(43), None),
        ],
        vesting: vec![(eth(&alice()), (50, 10, 1))],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    t.into()
}
