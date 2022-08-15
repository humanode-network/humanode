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

fn mocks_lock() -> std::sync::MutexGuard<'static, ()> {
    static MOCK_RUNTIME_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    // Ignore the poisoning for the tests that panic.
    // We only care about concurrency here, not about the poisoning.
    match MOCK_RUNTIME_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub fn with_mocks_lock<R>(f: impl FnOnce() -> R) -> R {
    let lock = mocks_lock();
    let res = f();
    drop(lock);
    res
}
