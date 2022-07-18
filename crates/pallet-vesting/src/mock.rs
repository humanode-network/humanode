use std::cell::RefCell;

use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64, GenesisBuild},
};
use mockall::mock;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, Identity, IdentityLookup},
};

use super::*;
use crate as pallet_vesting;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

/// A timestamp: milliseconds since the unix epoch.
pub type UnixMilliseconds = u64;

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

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Vesting: pallet_vesting::{Pallet, Call, Storage, Event<T>, Config<T>},
    }
);

parameter_types! {
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
}
impl frame_system::Config for Test {
    type AccountData = pallet_balances::AccountData<u64>;
    type AccountId = u64;
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockHashCount = ConstU64<250>;
    type BlockLength = ();
    type BlockNumber = u64;
    type BlockWeights = ();
    type Call = Call;
    type DbWeight = ();
    type Event = Event;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Index = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type Origin = Origin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = ();
    type SystemWeightInfo = ();
    type Version = ();
}

impl pallet_balances::Config for Test {
    type AccountStore = System;
    type Balance = u64;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = ConstU32<10>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}
parameter_types! {
    pub const MinVestedTransfer: u64 = 256 * 2;
    pub static ExistentialDeposit: u64 = 0;
}
impl Config for Test {
    type Event = Event;
    type Currency = Balances;
    type Moment = UnixMilliseconds;
    type CurrentMoment = MockCurrentMomentProvider;
    type MomentToBalance = Identity;
    const MAX_VESTING_SCHEDULES: u32 = 3;
    type MinVestedTransfer = MinVestedTransfer;
    type WeightInfo = ();
}

pub struct ExtBuilder {
    existential_deposit: u64,
    vesting_genesis_config: Option<Vec<(u64, UnixMilliseconds, UnixMilliseconds, u64)>>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            existential_deposit: 1,
            vesting_genesis_config: None,
        }
    }
}

impl ExtBuilder {
    pub fn existential_deposit(mut self, existential_deposit: u64) -> Self {
        self.existential_deposit = existential_deposit;
        self
    }

    pub fn vesting_genesis_config(
        mut self,
        config: Vec<(u64, UnixMilliseconds, UnixMilliseconds, u64)>,
    ) -> Self {
        self.vesting_genesis_config = Some(config);
        self
    }

    pub fn build(self) -> sp_io::TestExternalities {
        EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![
                (1, 10 * self.existential_deposit),
                (2, 20 * self.existential_deposit),
                (3, 30 * self.existential_deposit),
                (4, 40 * self.existential_deposit),
                (12, 10 * self.existential_deposit),
                (13, 9999 * self.existential_deposit),
            ],
        }
        .assimilate_storage(&mut t)
        .unwrap();

        let vesting = if let Some(vesting_config) = self.vesting_genesis_config {
            vesting_config
        } else {
            vec![
                (1, 5, 10, 5 * self.existential_deposit),
                (2, 10, 20, 0),
                (12, 5, 20, 5 * self.existential_deposit),
            ]
        };

        pallet_vesting::GenesisConfig::<Test> { vesting }
            .assimilate_storage(&mut t)
            .unwrap();
        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}
