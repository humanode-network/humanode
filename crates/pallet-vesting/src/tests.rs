//! The tests for the pallet.

use frame_support::{assert_noop, assert_ok, sp_runtime::DispatchError};
use mockall::predicate;

use crate::{
    mock::{
        new_test_ext, Balances, MockSchedule, MockSchedulingDriver, Origin, Test,
        TestExternalitiesExt, Vesting,
    },
    *,
};

/// This test verifies that `lock_under_vesting` works in the happy path.
#[test]
fn lock_under_vesting_works() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare the test state.
        Balances::make_free_balance_be(&42, 1000);

        // Check test preconditions.
        assert_eq!(Balances::free_balance(&42), 1000);
        assert_eq!(Balances::usable_balance(&42), 1000);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx.expect().never();

        // Invoke the function under test.
        assert_ok!(Vesting::lock_under_vesting(&42, 100, MockSchedule));

        // Assert state changes.
        assert_eq!(Balances::free_balance(&42), 1000);
        assert_eq!(Balances::usable_balance(&42), 900);

        // Assert mock invocations.
        compute_balance_under_lock_ctx.checkpoint();
    });
}

/// This test verifies that `lock_under_vesting` does not allow engaging a lock if there is another
/// lock already present.
#[test]
fn lock_under_vesting_conflicts_with_existing_lock() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare the test state.
        Balances::make_free_balance_be(&42, 1000);
        assert_ok!(Vesting::lock_under_vesting(&42, 100, MockSchedule));

        // Check test preconditions.
        assert_eq!(Balances::free_balance(&42), 1000);
        assert_eq!(Balances::usable_balance(&42), 900);
        assert!(<Locks<Test>>::contains_key(&42));

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx.expect().never();

        // Invoke the function under test.
        assert_noop!(
            Vesting::lock_under_vesting(&42, 100, MockSchedule),
            <Error<Test>>::VestingAlreadyEngaged
        );

        // Assert state changes.
        assert_eq!(Balances::free_balance(&42), 1000);
        assert_eq!(Balances::usable_balance(&42), 900);

        // Assert mock invocations.
        compute_balance_under_lock_ctx.checkpoint();
    });
}

/// This test verifies that `unlock` works in the happy path when we need to unlock the whole
/// balance.
#[test]
fn unlock_works_full() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare the test state.
        Balances::make_free_balance_be(&42, 1000);
        assert_ok!(Vesting::lock_under_vesting(&42, 100, MockSchedule));

        // Check test preconditions.
        assert!(<Locks<Test>>::contains_key(&42));
        assert_eq!(Balances::free_balance(&42), 1000);
        assert_eq!(Balances::usable_balance(&42), 900);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx
            .expect()
            .once()
            .with(predicate::eq(100), predicate::eq(MockSchedule))
            .return_const(Ok(0));

        // Invoke the function under test.
        assert_ok!(Vesting::unlock(Origin::signed(42)));

        // Assert state changes.
        assert_eq!(Balances::free_balance(&42), 1000);
        assert_eq!(Balances::usable_balance(&42), 1000);
        assert!(!<Locks<Test>>::contains_key(&42));

        // Assert mock invocations.
        compute_balance_under_lock_ctx.checkpoint();
    });
}

/// This test verifies that `unlock` works in the happy path when we need to unlock a fraction
/// of the balance.
#[test]
fn unlock_works_partial() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare the test state.
        Balances::make_free_balance_be(&42, 1000);
        assert_ok!(Vesting::lock_under_vesting(&42, 100, MockSchedule));

        // Check test preconditions.
        let lock_before = <Locks<Test>>::get(&42).unwrap();
        assert_eq!(Balances::free_balance(&42), 1000);
        assert_eq!(Balances::usable_balance(&42), 900);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx
            .expect()
            .once()
            .with(predicate::eq(100), predicate::eq(MockSchedule))
            .return_const(Ok(90));

        // Invoke the function under test.
        assert_ok!(Vesting::unlock(Origin::signed(42)));

        // Assert state changes.
        assert_eq!(Balances::free_balance(&42), 1000);
        assert_eq!(Balances::usable_balance(&42), 910);
        assert_eq!(<Locks<Test>>::get(&42).unwrap(), lock_before);

        // Assert mock invocations.
        compute_balance_under_lock_ctx.checkpoint();
    });
}

/// This test verifies that `unlock` results in a valid state after the scheduling driver
/// computation has failed.
#[test]
fn unlock_computation_failure() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare the test state.
        Balances::make_free_balance_be(&42, 1000);
        assert_ok!(Vesting::lock_under_vesting(&42, 100, MockSchedule));

        // Check test preconditions.
        let lock_before = <Locks<Test>>::get(&42).unwrap();
        assert_eq!(Balances::free_balance(&42), 1000);
        assert_eq!(Balances::usable_balance(&42), 900);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx
            .expect()
            .once()
            .with(predicate::eq(100), predicate::eq(MockSchedule))
            .return_const(Err(DispatchError::Other("compute_balance_under failed")));

        // Invoke the function under test.
        assert_noop!(
            Vesting::unlock(Origin::signed(42)),
            DispatchError::Other("compute_balance_under failed")
        );

        // Assert state changes.
        assert_eq!(Balances::free_balance(&42), 1000);
        assert_eq!(Balances::usable_balance(&42), 900);
        assert_eq!(<Locks<Test>>::get(&42).unwrap(), lock_before);

        // Assert mock invocations.
        compute_balance_under_lock_ctx.checkpoint();
    });
}
