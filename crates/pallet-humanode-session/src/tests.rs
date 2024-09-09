//! The tests for the pallet.

use std::collections::BTreeSet;

use frame_support::{assert_noop, assert_ok};
use mock::{HumanodeSession, RuntimeOrigin};
use sp_runtime::BoundedBTreeSet;

use crate::{
    mock::{new_test_ext, Test, TestExternalitiesExt},
    *,
};

/// This test verifies that basic setup works in the happy path.
#[test]
fn basic_setup_works() {
    new_test_ext().execute_with_ext(|_| {
        assert!(<BannedAccounts<Test>>::get().is_empty());
    });
}

/// This test verifies that ban works in the happy path.
#[test]
fn ban_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(<BannedAccounts<Test>>::get().is_empty());

        // Invoke the function under test.
        assert_ok!(HumanodeSession::ban(RuntimeOrigin::root(), 1));

        // Assert state changes.
        assert_eq!(
            <BannedAccounts<Test>>::get().into_inner(),
            vec![1].into_iter().collect::<_>()
        );
    });
}

/// This test prevents ban when the provided account is related to bootnodes.
#[test]
fn ban_fails_attempt_to_ban_bootnode() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(<BannedAccounts<Test>>::get().is_empty());

        // Invoke the function under test.
        assert_noop!(
            HumanodeSession::ban(RuntimeOrigin::root(), 42),
            Error::<Test>::AttemptToBanBootnode
        );
    });
}

/// This test prevents ban when the banned accounts limit has been reached as `BoundedBTreeSet`.
#[test]
fn ban_fails_too_many_banned_accounts() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test preconditions.
        let banned_accounts_before = vec![1, 2, 3, 4, 5].into_iter().collect::<BTreeSet<_>>();
        <BannedAccounts<Test>>::put(BoundedBTreeSet::try_from(banned_accounts_before).unwrap());

        // Invoke the function under test.
        assert_noop!(
            HumanodeSession::ban(RuntimeOrigin::root(), 6),
            Error::<Test>::TooManyBannedAccounts
        );
    });
}

/// This test prevents ban when the provided account is already banned.
#[test]
fn ban_fails_account_is_already_banned() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test preconditions.
        let banned_accounts_before = vec![1, 2, 3].into_iter().collect::<BTreeSet<_>>();
        <BannedAccounts<Test>>::put(BoundedBTreeSet::try_from(banned_accounts_before).unwrap());

        // Invoke the function under test.
        assert_noop!(
            HumanodeSession::ban(RuntimeOrigin::root(), 3),
            Error::<Test>::AccountIsAlreadyBanned
        );
    });
}

/// This test verifies that unban works in the happy path.
#[test]
fn unban_works() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test preconditions.
        let banned_accounts_before = vec![1, 2, 3, 4].into_iter().collect::<BTreeSet<_>>();
        <BannedAccounts<Test>>::put(BoundedBTreeSet::try_from(banned_accounts_before).unwrap());

        // Invoke the function under test.
        assert_ok!(HumanodeSession::unban(RuntimeOrigin::root(), 1));

        // Assert state changes.
        assert_eq!(
            <BannedAccounts<Test>>::get().into_inner(),
            vec![2, 3, 4].into_iter().collect::<_>()
        );
    });
}

/// This test prevents unban when the provided account is not banned.
#[test]
fn unban_fails_account_is_not_banned() {
    new_test_ext().execute_with_ext(|_| {
        // Invoke the function under test.
        assert_noop!(
            HumanodeSession::unban(RuntimeOrigin::root(), 1),
            Error::<Test>::AccountIsNotBanned
        );
    });
}
