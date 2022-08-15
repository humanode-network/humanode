//! The benchmarks for the pallet.

use frame_benchmarking::benchmarks;
use frame_support::traits::{ExistenceRequirement, WithdrawReasons};
use frame_system::RawOrigin;

use crate::*;

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
            T: Interface
    }

    unlock {
        let account_id = <T as Interface>::account_id();
        let schedule = <T as Interface>::schedule();

        let imbalance = <CurrencyOf<T>>::deposit_creating(&account_id, 1000u32.into());
        assert_eq!(<CurrencyOf<T>>::free_balance(&account_id), 1000u32.into());
        assert!(<CurrencyOf<T>>::ensure_can_withdraw(&account_id, 1000u32.into(), WithdrawReasons::empty(), 0u32.into()).is_ok());

        #[cfg(test)]
        let test_data = {
            use crate::mock;

            let mock_runtime_guard = mock::runtime_lock();

            let compute_balance_under_lock_ctx = mock::MockSchedulingDriver::compute_balance_under_lock_context();
            compute_balance_under_lock_ctx.expect().once().return_const(Ok(100));

            (mock_runtime_guard, compute_balance_under_lock_ctx)
        };

        <Pallet<T>>::lock_under_vesting(&account_id, schedule)?;
        assert_eq!(<CurrencyOf<T>>::free_balance(&account_id), 1000u32.into());
        assert!(<CurrencyOf<T>>::ensure_can_withdraw(&account_id, 1000u32.into(), WithdrawReasons::empty(), 0u32.into()).is_err());

        #[cfg(test)]
        let test_data = {
            let (mock_runtime_guard, compute_balance_under_lock_ctx) = test_data;

            compute_balance_under_lock_ctx.expect().times(1..).return_const(Ok(0));

            (mock_runtime_guard, compute_balance_under_lock_ctx)
        };

        let origin = RawOrigin::Signed(account_id.clone());

    }: _(origin)
    verify {
        assert_eq!(Schedules::<T>::get(&account_id), None);
        assert_eq!(<CurrencyOf<T>>::free_balance(&account_id), 1000u32.into());
        assert!(<CurrencyOf<T>>::ensure_can_withdraw(&account_id, 1000u32.into(), WithdrawReasons::empty(), 0u32.into()).is_ok());

        #[cfg(test)]
        {
            let (mock_runtime_guard, compute_balance_under_lock_ctx) = test_data;

            compute_balance_under_lock_ctx.checkpoint();

            drop(mock_runtime_guard);
        }

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
