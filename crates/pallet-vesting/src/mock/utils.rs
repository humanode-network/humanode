//! Mock utils.

// Allow simple integer arithmetic in tests.
#![allow(clippy::integer_arithmetic)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{sp_runtime::DispatchError, Deserialize, Serialize};
use mockall::mock;
use scale_info::TypeInfo;

use super::*;
use crate::traits;

#[derive(
    Debug, Clone, Decode, Encode, MaxEncodedLen, TypeInfo, PartialEq, Eq, Serialize, Deserialize,
)]
pub struct MockSchedule;

mock! {
    #[derive(Debug)]
    pub SchedulingDriver {}
    impl traits::SchedulingDriver for SchedulingDriver {
        type Balance = crate::BalanceOf<Test>;
        type Schedule = MockSchedule;

        fn compute_balance_under_lock(
            schedule: &<Self as traits::SchedulingDriver>::Schedule,
        ) -> Result<<Self as traits::SchedulingDriver>::Balance, DispatchError>;
    }
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
