//! The benchmarks for the pallet.

// Allow integer and float arithmetic in tests.
#![allow(clippy::arithmetic_side_effects, clippy::float_arithmetic)]

use frame_benchmarking::benchmarks;
use frame_support::{
    assert_ok,
    dispatch::DispatchResult,
    traits::{ExistenceRequirement, WithdrawReasons},
};
use frame_system::RawOrigin;

use crate::*;

/// The benchmarking extension for the scheduling driver.
pub trait SchedulingDriver: traits::SchedulingDriver {
    /// The data to be passed from `prepare_init` to `verify` flow.
    type Data;

    /// Initialize the scheduling driver environment to a state where vesting creation is possible.
    fn prepare_init() -> Self::Data;
    /// Advance the scheduling driver environment to a state where unlocking is possible.
    fn prepare_advance(data: Self::Data) -> Self::Data;
    /// Verify scheduling driver environment after the benchmark.
    fn verify(data: Self::Data) -> DispatchResult;
}

/// The benchmark interface into the environment.
pub trait Interface: super::Config {
    /// Obtain an Account ID.
    ///
    /// This is an account to unlock the vested balance for.
    fn account_id() -> <Self as frame_system::Config>::AccountId;

    /// Obtain the vesting schedule.
    ///
    /// This is the vesting.
    fn schedule() -> <Self as Config>::Schedule;
}

benchmarks! {
    where_clause {
        where
            T: Interface,
            <T as super::Config>::SchedulingDriver: SchedulingDriver,
    }

    unlock {
        let account_id = <T as Interface>::account_id();
        let schedule = <T as Interface>::schedule();
        let init_balance = <CurrencyOf<T>>::total_balance(&account_id);

        let imbalance = <CurrencyOf<T>>::deposit_creating(&account_id, 1000u32.into());
        assert_eq!(<CurrencyOf<T>>::free_balance(&account_id), init_balance + 1000u32.into());
        assert!(<CurrencyOf<T>>::ensure_can_withdraw(&account_id, init_balance + 1000u32.into(), WithdrawReasons::empty(), 0u32.into()).is_ok());

        let scheduling_driver = <T as super::Config>::SchedulingDriver::prepare_init();

        <Pallet<T>>::lock_under_vesting(&account_id, schedule)?;
        assert_eq!(<CurrencyOf<T>>::free_balance(&account_id), init_balance + 1000u32.into());
        assert!(<CurrencyOf<T>>::ensure_can_withdraw(&account_id, init_balance + 1000u32.into(), WithdrawReasons::empty(), 0u32.into()).is_err());

        let scheduling_driver = <T as super::Config>::SchedulingDriver::prepare_advance(scheduling_driver);

        let origin = RawOrigin::Signed(account_id.clone());

    }: _(origin)
    verify {
        assert_eq!(Schedules::<T>::get(&account_id), None);
        assert_eq!(<CurrencyOf<T>>::free_balance(&account_id), init_balance + 1000u32.into());
        assert!(<CurrencyOf<T>>::ensure_can_withdraw(&account_id, init_balance + 1000u32.into(), WithdrawReasons::empty(), 0u32.into()).is_ok());

        assert_ok!(<T as super::Config>::SchedulingDriver::verify(scheduling_driver));

        // Clean up imbalance after ourselves.
        <CurrencyOf<T>>::settle(&account_id, imbalance, WithdrawReasons::RESERVE, ExistenceRequirement::AllowDeath).ok().unwrap();
    }

    update_schedule {
        let account_id = <T as Interface>::account_id();
        let schedule = <T as Interface>::schedule();
        let init_balance = <CurrencyOf<T>>::total_balance(&account_id);

        let imbalance = <CurrencyOf<T>>::deposit_creating(&account_id, 1000u32.into());
        assert_eq!(<CurrencyOf<T>>::free_balance(&account_id), init_balance + 1000u32.into());
        assert!(<CurrencyOf<T>>::ensure_can_withdraw(&account_id, init_balance + 1000u32.into(), WithdrawReasons::empty(), 0u32.into()).is_ok());

        let scheduling_driver = <T as super::Config>::SchedulingDriver::prepare_init();

        <Pallet<T>>::lock_under_vesting(&account_id, schedule.clone())?;
        assert_eq!(<CurrencyOf<T>>::free_balance(&account_id), init_balance + 1000u32.into());
        assert!(<CurrencyOf<T>>::ensure_can_withdraw(&account_id, init_balance + 1000u32.into(), WithdrawReasons::empty(), 0u32.into()).is_err());

        // Update vesting with full unlock.
        let scheduling_driver = <T as super::Config>::SchedulingDriver::prepare_advance(scheduling_driver);

        let origin = RawOrigin::Root;

    }: _(origin, account_id.clone(), schedule)
    verify {
        assert_eq!(Schedules::<T>::get(&account_id), None);
        assert_eq!(<CurrencyOf<T>>::free_balance(&account_id), init_balance + 1000u32.into());
        assert!(<CurrencyOf<T>>::ensure_can_withdraw(&account_id, init_balance + 1000u32.into(), WithdrawReasons::empty(), 0u32.into()).is_ok());

        assert_ok!(<T as super::Config>::SchedulingDriver::verify(scheduling_driver));

        // Clean up imbalance after ourselves.
        <CurrencyOf<T>>::settle(&account_id, imbalance, WithdrawReasons::RESERVE, ExistenceRequirement::AllowDeath).ok().unwrap();
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test,
    );
}

#[cfg(test)]
impl Interface for crate::mock::Test {
    fn account_id() -> <Self as frame_system::Config>::AccountId {
        42
    }

    fn schedule() -> <Self as Config>::Schedule {
        mock::MockSchedule
    }
}

#[cfg(test)]
impl SchedulingDriver for <crate::mock::Test as super::Config>::SchedulingDriver {
    type Data = (
        std::sync::MutexGuard<'static, ()>,
        mock::__mock_MockSchedulingDriver_SchedulingDriver::__compute_balance_under_lock::Context,
    );

    fn prepare_init() -> Self::Data {
        let mock_runtime_guard = mock::runtime_lock();

        let compute_balance_under_lock_ctx =
            mock::MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx
            .expect()
            .once()
            .return_const(Ok(100));

        (mock_runtime_guard, compute_balance_under_lock_ctx)
    }

    fn prepare_advance(data: Self::Data) -> Self::Data {
        let (mock_runtime_guard, compute_balance_under_lock_ctx) = data;

        compute_balance_under_lock_ctx.checkpoint();

        compute_balance_under_lock_ctx
            .expect()
            .times(1..)
            .return_const(Ok(0));

        (mock_runtime_guard, compute_balance_under_lock_ctx)
    }

    fn verify(data: Self::Data) -> DispatchResult {
        let (mock_runtime_guard, compute_balance_under_lock_ctx) = data;

        compute_balance_under_lock_ctx.checkpoint();

        drop(mock_runtime_guard);
        Ok(())
    }
}
