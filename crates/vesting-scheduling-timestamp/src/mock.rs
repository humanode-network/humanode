use frame_support::traits::ConstU32;
use mockall::mock;
use vesting_schedule_linear::traits::SimpleFracScaler;

use super::*;

pub type Driver = Adapter<Test, MultiLinearScheduleOf<Test>>;

pub enum Test {}

impl Config for Test {
    type Balance = u8;
    type Timestamp = u8;
    type StartingPoint = MockStartingPoint;
    type Now = MockNow;
}

impl LinearScheduleConfig for Test {
    type FracScale = SimpleFracScaler<u16, <Self as Config>::Balance, <Self as Config>::Timestamp>;
}

impl MultiLinearScheduleConfig for Test {
    type MaxSchedulesPerAccount = ConstU32<5>;
}

mock! {
    pub StartingPoint {}
    impl Get<Option<u8>> for StartingPoint {
        fn get() -> Option<u8>;
    }
}

mock! {
    pub Now {}
    impl Get<u8> for Now {
        fn get() -> u8;
    }
}
