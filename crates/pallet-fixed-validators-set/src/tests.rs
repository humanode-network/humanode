//! The tests for the pallet.

// Allow simple integer arithmetic in tests.
#![allow(clippy::arithmetic_side_effects)]

use frame_support::{assert_noop, assert_ok, BoundedVec};

use crate::{
    mock::{new_test_ext, FixedValidatorSet, RuntimeOrigin, Test},
    *,
};

/// This test verifies that the test setup for the genesis is adequate.
#[test]
fn basic_setup_works() {
    new_test_ext().execute_with(|| {
        // Check the initial set.
        assert_eq!(<Validators<Test>>::get(), vec![]);
    });
}

/// This test verifies that updating the set works in the happy path.
#[test]
fn updating_set_works() {
    new_test_ext().execute_with(|| {
        // Check test preconditions.
        assert_eq!(<Validators<Test>>::get(), vec![]);

        // Set block number to enable events.
        mock::System::set_block_number(1);

        // Invoke the function under test.
        assert_ok!(FixedValidatorSet::update_set(
            RuntimeOrigin::root(),
            BoundedVec::try_from(vec![1]).unwrap()
        ));

        // Assert state changes.
        assert_eq!(<Validators<Test>>::get(), vec![1]);
        mock::System::assert_has_event(mock::RuntimeEvent::FixedValidatorSet(Event::SetUpdated));
    });
}

/// This test verifies that updating the set works fail with a non-root origin.
#[test]
fn updating_set_requires_root() {
    new_test_ext().execute_with(|| {
        // Check test preconditions.
        assert_eq!(<Validators<Test>>::get(), vec![]);

        // Invoke the function under test and assert it fails.
        assert_noop!(
            FixedValidatorSet::update_set(
                RuntimeOrigin::signed(1),
                BoundedVec::try_from(vec![1]).unwrap()
            ),
            frame_support::dispatch::DispatchError::BadOrigin
        );
        assert_noop!(
            FixedValidatorSet::update_set(
                RuntimeOrigin::none(),
                BoundedVec::try_from(vec![1]).unwrap(),
            ),
            frame_support::dispatch::DispatchError::BadOrigin
        );
    });
}
