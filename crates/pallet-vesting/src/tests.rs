//! The tests for the pallet.

use frame_support::{assert_noop, assert_ok, assert_storage_noop, sp_runtime::DispatchError};
use mockall::predicate;

use crate::{
    mock::{
        new_test_ext, Balances, MockSchedule, MockSchedulingDriver, RuntimeOrigin, System, Test,
        TestExternalitiesExt, Vesting,
    },
    *,
};

/// This test verifies that `lock_under_vesting` works in the happy path (with non-zero balance).
#[test]
fn lock_under_vesting_works() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare the test state.
        Balances::make_free_balance_be(&42, 1000);

        // Check test preconditions.
        assert!(<Schedules<Test>>::get(42).is_none());
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 1000);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx
            .expect()
            .once()
            .with(predicate::eq(MockSchedule))
            .return_const(Ok(100));

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_ok!(Vesting::lock_under_vesting(&42, MockSchedule));

        // Assert state changes.
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 900);
        assert!(<Schedules<Test>>::get(42).is_some());
        assert_eq!(System::events().len(), 3);
        System::assert_has_event(mock::RuntimeEvent::Vesting(Event::Locked {
            who: 42,
            schedule: MockSchedule,
            balance_under_lock: 100,
        }));
        System::assert_has_event(mock::RuntimeEvent::Balances(
            pallet_balances::Event::Locked {
                who: 42,
                amount: 100,
            },
        ));
        System::assert_has_event(mock::RuntimeEvent::Vesting(Event::PartiallyUnlocked {
            who: 42,
            balance_left_under_lock: 100,
        }));

        // Assert mock invocations.
        compute_balance_under_lock_ctx.checkpoint();
    });
}

/// This test verifies that `lock_under_vesting` works in the happy path (with zero balance).
#[test]
fn lock_under_vesting_works_with_zero() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare the test state.
        Balances::make_free_balance_be(&42, 1000);

        // Check test preconditions.
        assert!(<Schedules<Test>>::get(42).is_none());
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 1000);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx
            .expect()
            .once()
            .with(predicate::eq(MockSchedule))
            .return_const(Ok(0));

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_ok!(Vesting::lock_under_vesting(&42, MockSchedule));

        // Assert state changes.
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 1000);
        assert!(<Schedules<Test>>::get(42).is_none());
        assert_eq!(System::events().len(), 2);
        System::assert_has_event(mock::RuntimeEvent::Vesting(Event::Locked {
            who: 42,
            schedule: MockSchedule,
            balance_under_lock: 0,
        }));
        System::assert_has_event(mock::RuntimeEvent::Vesting(Event::FullyUnlocked {
            who: 42,
        }));

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
        <Pallet<Test>>::set_lock(&42, 100);
        <Schedules<Test>>::insert(42, MockSchedule);

        // Check test preconditions.
        let schedule_before = <Schedules<Test>>::get(42).unwrap();
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 900);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx.expect().never();

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_noop!(
            Vesting::lock_under_vesting(&42, MockSchedule),
            <Error<Test>>::VestingAlreadyEngaged
        );

        // Assert state changes.
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 900);
        assert_eq!(<Schedules<Test>>::get(42).unwrap(), schedule_before);
        assert_eq!(System::events().len(), 0);

        // Assert mock invocations.
        compute_balance_under_lock_ctx.checkpoint();
    });
}

/// This test verifies that `lock_under_vesting` can lock the balance greater than the free balance
/// available at the account.
/// This is not a part of the design, but just demonstrates this one property of the system we have
/// here.
#[test]
fn lock_under_vesting_can_lock_balance_greater_than_free_balance() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare the test state.
        Balances::make_free_balance_be(&42, 1000);

        // Check test preconditions.
        assert!(<Schedules<Test>>::get(42).is_none());
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 1000);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx
            .expect()
            .once()
            .with(predicate::eq(MockSchedule))
            .return_const(Ok(1100));

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_ok!(Vesting::lock_under_vesting(&42, MockSchedule));

        // Assert state changes.
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 0);
        assert!(<Schedules<Test>>::get(42).is_some());
        assert_eq!(System::events().len(), 3);
        System::assert_has_event(mock::RuntimeEvent::Vesting(Event::Locked {
            who: 42,
            schedule: MockSchedule,
            balance_under_lock: 1100,
        }));
        System::assert_has_event(mock::RuntimeEvent::Balances(
            pallet_balances::Event::Locked {
                who: 42,
                amount: 1100,
            },
        ));
        System::assert_has_event(mock::RuntimeEvent::Vesting(Event::PartiallyUnlocked {
            who: 42,
            balance_left_under_lock: 1100,
        }));

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
        <Pallet<Test>>::set_lock(&42, 100);
        <Schedules<Test>>::insert(42, MockSchedule);

        // Check test preconditions.
        assert!(<Schedules<Test>>::get(42).is_some());
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 900);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx
            .expect()
            .once()
            .with(predicate::eq(MockSchedule))
            .return_const(Ok(0));

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_ok!(Vesting::unlock(RuntimeOrigin::signed(42)));

        // Assert state changes.
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 1000);
        assert!(<Schedules<Test>>::get(42).is_none());
        assert_eq!(System::events().len(), 2);
        System::assert_has_event(mock::RuntimeEvent::Vesting(Event::FullyUnlocked {
            who: 42,
        }));
        System::assert_has_event(mock::RuntimeEvent::Balances(
            pallet_balances::Event::Unlocked {
                who: 42,
                amount: 100,
            },
        ));

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
        <Pallet<Test>>::set_lock(&42, 100);
        <Schedules<Test>>::insert(42, MockSchedule);

        // Check test preconditions.
        let schedule_before = <Schedules<Test>>::get(42).unwrap();
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 900);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx
            .expect()
            .once()
            .with(predicate::eq(MockSchedule))
            .return_const(Ok(90));

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_ok!(Vesting::unlock(RuntimeOrigin::signed(42)));

        // Assert state changes.
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 910);
        assert_eq!(<Schedules<Test>>::get(42).unwrap(), schedule_before);
        assert_eq!(System::events().len(), 2);
        System::assert_has_event(mock::RuntimeEvent::Vesting(Event::PartiallyUnlocked {
            who: 42,
            balance_left_under_lock: 90,
        }));
        System::assert_has_event(mock::RuntimeEvent::Balances(
            pallet_balances::Event::Unlocked {
                who: 42,
                amount: 10,
            },
        ));

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
        <Pallet<Test>>::set_lock(&42, 100);
        <Schedules<Test>>::insert(42, MockSchedule);

        // Check test preconditions.
        let schedule_before = <Schedules<Test>>::get(42).unwrap();
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 900);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx
            .expect()
            .once()
            .with(predicate::eq(MockSchedule))
            .return_const(Err(DispatchError::Other("compute_balance_under failed")));

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_noop!(
            Vesting::unlock(RuntimeOrigin::signed(42)),
            DispatchError::Other("compute_balance_under failed")
        );

        // Assert state changes.
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 900);
        assert_eq!(<Schedules<Test>>::get(42).unwrap(), schedule_before);
        assert_eq!(System::events().len(), 0);

        // Assert mock invocations.
        compute_balance_under_lock_ctx.checkpoint();
    });
}

/// This test verifies the `unlock` behaviour when it is called for an account that does not
/// have vesting.
#[test]
fn unlock_no_vesting_error() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare the test state.
        Balances::make_free_balance_be(&42, 1000);

        // Check test preconditions.
        assert!(<Schedules<Test>>::get(42).is_none());
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 1000);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx.expect().never();

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_noop!(
            Vesting::unlock(RuntimeOrigin::signed(42)),
            <Error<Test>>::NoVesting,
        );

        // Assert state changes.
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 1000);
        assert!(<Schedules<Test>>::get(42).is_none());
        assert_eq!(System::events().len(), 0);

        // Assert mock invocations.
        compute_balance_under_lock_ctx.checkpoint();
    });
}

/// This test verifies that `update_schedule` works in the happy path (with non-zero balance).
#[test]
fn update_vesting_works_partial_unlock() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare the test state.
        Balances::make_free_balance_be(&42, 1000);
        <Pallet<Test>>::set_lock(&42, 100);
        <Schedules<Test>>::insert(42, MockSchedule);

        // Check test preconditions.
        assert!(<Schedules<Test>>::get(42).is_some());
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 900);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx
            .expect()
            .once()
            .with(predicate::eq(MockSchedule))
            .return_const(Ok(50));

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_ok!(Vesting::update_schedule(
            RuntimeOrigin::root(),
            42,
            MockSchedule
        ));

        // Assert state changes.
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 950);
        assert!(<Schedules<Test>>::get(42).is_some());
        assert_eq!(System::events().len(), 3);
        System::assert_has_event(mock::RuntimeEvent::Vesting(Event::VestingUpdated {
            account_id: 42,
            old_schedule: MockSchedule,
            new_schedule: MockSchedule,
            balance_under_lock: 50,
        }));
        System::assert_has_event(mock::RuntimeEvent::Vesting(Event::PartiallyUnlocked {
            who: 42,
            balance_left_under_lock: 50,
        }));
        System::assert_has_event(mock::RuntimeEvent::Balances(
            pallet_balances::Event::Unlocked {
                who: 42,
                amount: 50,
            },
        ));

        // Assert mock invocations.
        compute_balance_under_lock_ctx.checkpoint();
    });
}

/// This test verifies that `update_schedule` works in the happy path (with zero balance).
#[test]
fn update_vesting_works_full_unlock() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare the test state.
        Balances::make_free_balance_be(&42, 1000);
        <Pallet<Test>>::set_lock(&42, 100);
        <Schedules<Test>>::insert(42, MockSchedule);

        // Check test preconditions.
        assert!(<Schedules<Test>>::get(42).is_some());
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 900);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx
            .expect()
            .once()
            .with(predicate::eq(MockSchedule))
            .return_const(Ok(0));

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_ok!(Vesting::update_schedule(
            RuntimeOrigin::root(),
            42,
            MockSchedule
        ));

        // Assert state changes.
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 1000);
        assert!(<Schedules<Test>>::get(42).is_none());
        assert_eq!(System::events().len(), 3);
        System::assert_has_event(mock::RuntimeEvent::Vesting(Event::VestingUpdated {
            account_id: 42,
            old_schedule: MockSchedule,
            new_schedule: MockSchedule,
            balance_under_lock: 0,
        }));
        System::assert_has_event(mock::RuntimeEvent::Vesting(Event::FullyUnlocked {
            who: 42,
        }));
        System::assert_has_event(mock::RuntimeEvent::Balances(
            pallet_balances::Event::Unlocked {
                who: 42,
                amount: 100,
            },
        ));

        // Assert mock invocations.
        compute_balance_under_lock_ctx.checkpoint();
    });
}

/// This test verifies the `update_schedule` behaviour when it is called for an account that does not
/// have vesting.
#[test]
fn update_vesting_no_vesting_error() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare the test state.
        Balances::make_free_balance_be(&42, 1000);

        // Check test preconditions.
        assert!(<Schedules<Test>>::get(42).is_none());
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 1000);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx.expect().never();

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_noop!(
            Vesting::update_schedule(RuntimeOrigin::root(), 42, MockSchedule),
            <Error<Test>>::NoVesting,
        );

        // Assert state changes.
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 1000);
        assert!(<Schedules<Test>>::get(42).is_none());
        assert_eq!(System::events().len(), 0);

        // Assert mock invocations.
        compute_balance_under_lock_ctx.checkpoint();
    });
}

/// This test verifies that `update_schedule` signed by account different from sudo fails.
#[test]
fn update_vesting_not_sudo_error() {
    new_test_ext().execute_with_ext(|_| {
        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx.expect().never();

        // Set block number to enable events.
        System::set_block_number(1);

        // Non-sudo accounts are not allowed.
        assert_noop!(
            Vesting::update_schedule(RuntimeOrigin::signed(10), 42, MockSchedule),
            DispatchError::BadOrigin,
        );

        assert_eq!(System::events().len(), 0);

        // Assert mock invocations.
        compute_balance_under_lock_ctx.checkpoint();
    });
}

/// This test verifies that `evaluate_lock` works in the happy path when the logic evaluates
/// to unlock the whole balance.
#[test]
fn evaluate_lock_works_full() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare the test state.
        Balances::make_free_balance_be(&42, 1000);
        <Pallet<Test>>::set_lock(&42, 100);
        <Schedules<Test>>::insert(42, MockSchedule);

        // Check test preconditions.
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 900);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx
            .expect()
            .once()
            .with(predicate::eq(MockSchedule))
            .return_const(Ok(0));

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_storage_noop! {
            assert_eq!(Vesting::evaluate_lock(&42), Ok(0))
        }

        // Assert mock invocations.
        compute_balance_under_lock_ctx.checkpoint();
    });
}

/// This test verifies that `evaluate_lock` works in the happy path when the logic evaluates
/// to unlock a fraction of the balance.
#[test]
fn evaluate_lock_works_partial() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare the test state.
        Balances::make_free_balance_be(&42, 1000);
        <Pallet<Test>>::set_lock(&42, 100);
        <Schedules<Test>>::insert(42, MockSchedule);

        // Check test preconditions.
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 900);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx
            .expect()
            .once()
            .with(predicate::eq(MockSchedule))
            .return_const(Ok(90));

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_storage_noop! {
            assert_eq!(Vesting::evaluate_lock(&42), Ok(90))
        }

        // Assert mock invocations.
        compute_balance_under_lock_ctx.checkpoint();
    });
}

/// This test verifies the `evaluate_lock` behaviour when it is called for an account that does not
/// have vesting.
#[test]
fn evaluate_lock_no_vesting_error() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare the test state.
        Balances::make_free_balance_be(&42, 1000);

        // Check test preconditions.
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 1000);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx.expect().never();

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_noop!(Vesting::evaluate_lock(&42), api::EvaluationError::NoVesting);

        // Assert mock invocations.
        compute_balance_under_lock_ctx.checkpoint();
    });
}

/// This test verifies the `evaluate_lock` behaviour when the scheduling driver computation fails.
#[test]
fn evaluate_lock_computation_failure() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare the test state.
        Balances::make_free_balance_be(&42, 1000);
        <Pallet<Test>>::set_lock(&42, 100);
        <Schedules<Test>>::insert(42, MockSchedule);

        // Check test preconditions.
        assert_eq!(Balances::free_balance(42), 1000);
        assert_eq!(Balances::usable_balance(42), 900);

        // Set mock expectations.
        let compute_balance_under_lock_ctx =
            MockSchedulingDriver::compute_balance_under_lock_context();
        compute_balance_under_lock_ctx
            .expect()
            .once()
            .with(predicate::eq(MockSchedule))
            .return_const(Err(DispatchError::Other("compute_balance_under failed")));

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_noop!(
            Vesting::evaluate_lock(&42),
            api::EvaluationError::Computation(DispatchError::Other("compute_balance_under failed"))
        );

        // Assert mock invocations.
        compute_balance_under_lock_ctx.checkpoint();
    });
}
