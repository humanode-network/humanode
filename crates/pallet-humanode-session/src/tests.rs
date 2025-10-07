//! The tests for the pallet.

// Allow simple integer arithmetic in tests.
#![allow(clippy::arithmetic_side_effects)]

use std::collections::BTreeSet;

use frame_support::{assert_noop, assert_ok, pallet_prelude::*};
use mockall::predicate;
use sp_runtime::{traits::BlockNumberProvider, BoundedBTreeSet};

use crate::{
    mock::{
        new_test_ext, Bioauth, HumanodeSession, MockShouldEndSession, RuntimeOrigin, Session,
        System, Test, TestExternalitiesExt,
    },
    *,
};

/// A helper function to rotate session.
fn rotate_session() {
    let next_block_number = System::current_block_number() + 1;
    let session_index_before = Session::current_index();

    // Set mock expectations.
    let should_end_session_ctx = MockShouldEndSession::should_end_session_context();
    should_end_session_ctx
        .expect()
        .once()
        .with(predicate::eq(next_block_number))
        .return_const(true);

    // Invoke new block initialization.
    Session::on_initialize(next_block_number);

    // Assert state changes.
    assert_eq!(Session::current_index(), session_index_before + 1);

    // Assert mock invocations.
    should_end_session_ctx.checkpoint();
}

/// This test verifies that basic setup works in the happy path.
#[test]
fn basic_setup_works() {
    new_test_ext().execute_with_ext(|_| {
        assert!(<BannedAccounts<Test>>::get().is_empty());
    });
}

/// This test verifies that ban works for the bioauth validators.
#[test]
fn ban_works_on_bioauth() {
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

/// This test verifies that ban works for the fixed validators set validators.
#[test]
fn ban_works_on_fixed_validators_set() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(<BannedAccounts<Test>>::get().is_empty());

        // Invoke the function under test.
        assert_ok!(HumanodeSession::ban(RuntimeOrigin::root(), 10_001));

        // Assert state changes.
        assert_eq!(
            <BannedAccounts<Test>>::get().into_inner(),
            vec![10_001].into_iter().collect::<_>()
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

/// This test verifies full ban and unban lifecycle logic.
#[test]
fn full_ban_unban_lifecycle() {
    new_test_ext().execute_with_ext(|_| {
        // Check initial state.
        assert!(<BannedAccounts<Test>>::get().is_empty());
        assert_eq!(
            Bioauth::active_authentications(),
            vec![pallet_bioauth::Authentication {
                public_key: 1,
                expires_at: 1000,
            }]
        );
        assert_eq!(Session::validators(), vec![42, 43, 44, 1, 10_001]);
        assert_eq!(
            Session::queued_keys(),
            vec![
                (42, sp_runtime::testing::UintAuthorityId(42)),
                (43, sp_runtime::testing::UintAuthorityId(43)),
                (44, sp_runtime::testing::UintAuthorityId(44)),
                (1, sp_runtime::testing::UintAuthorityId(1)),
                (10_001, sp_runtime::testing::UintAuthorityId(10_001)),
            ]
        );

        // Invoke the ban under test.
        assert_ok!(HumanodeSession::ban(RuntimeOrigin::root(), 1));

        // Assert state changes.
        assert_eq!(
            <BannedAccounts<Test>>::get().into_inner(),
            vec![1].into_iter().collect::<_>()
        );
        assert_eq!(
            Bioauth::active_authentications(),
            vec![pallet_bioauth::Authentication {
                public_key: 1,
                expires_at: 1000,
            }]
        );
        assert_eq!(Session::validators(), vec![42, 43, 44, 1, 10_001]);
        assert_eq!(
            Session::queued_keys(),
            vec![
                (42, sp_runtime::testing::UintAuthorityId(42)),
                (43, sp_runtime::testing::UintAuthorityId(43)),
                (44, sp_runtime::testing::UintAuthorityId(44)),
                (1, sp_runtime::testing::UintAuthorityId(1)),
                (10_001, sp_runtime::testing::UintAuthorityId(10_001)),
            ]
        );

        // Rotate session.
        rotate_session();

        // Assert state changes.
        //
        // Expect that the banned account hasn't been added to queued keys.
        assert_eq!(
            <BannedAccounts<Test>>::get().into_inner(),
            vec![1].into_iter().collect::<_>()
        );
        assert_eq!(
            Bioauth::active_authentications(),
            vec![pallet_bioauth::Authentication {
                public_key: 1,
                expires_at: 1000,
            }]
        );
        assert_eq!(Session::validators(), vec![42, 43, 44, 1, 10_001]);
        assert_eq!(
            Session::queued_keys(),
            vec![
                (42, sp_runtime::testing::UintAuthorityId(42)),
                (43, sp_runtime::testing::UintAuthorityId(43)),
                (44, sp_runtime::testing::UintAuthorityId(44)),
                (10_001, sp_runtime::testing::UintAuthorityId(10_001)),
            ]
        );

        // Rotate session.
        rotate_session();

        // Assert state changes.
        //
        // Expect that the banned account has been removed from validators list
        // and hasn't been added to queued keys.
        assert_eq!(
            <BannedAccounts<Test>>::get().into_inner(),
            vec![1].into_iter().collect::<_>()
        );
        assert_eq!(
            Bioauth::active_authentications(),
            vec![pallet_bioauth::Authentication {
                public_key: 1,
                expires_at: 1000,
            }]
        );
        assert_eq!(Session::validators(), vec![42, 43, 44, 10_001]);
        assert_eq!(
            Session::queued_keys(),
            vec![
                (42, sp_runtime::testing::UintAuthorityId(42)),
                (43, sp_runtime::testing::UintAuthorityId(43)),
                (44, sp_runtime::testing::UintAuthorityId(44)),
                (10_001, sp_runtime::testing::UintAuthorityId(10_001)),
            ]
        );

        // Invoke the unban under test.
        assert_ok!(HumanodeSession::unban(RuntimeOrigin::root(), 1));

        // Rotate session.
        rotate_session();

        // Assert state changes.
        //
        // Expect that the unbanned account has been added to queued keys.
        assert!(<BannedAccounts<Test>>::get().is_empty());
        assert_eq!(
            Bioauth::active_authentications(),
            vec![pallet_bioauth::Authentication {
                public_key: 1,
                expires_at: 1000,
            }]
        );
        assert_eq!(Session::validators(), vec![42, 43, 44, 10_001]);
        assert_eq!(
            Session::queued_keys(),
            vec![
                (42, sp_runtime::testing::UintAuthorityId(42)),
                (43, sp_runtime::testing::UintAuthorityId(43)),
                (44, sp_runtime::testing::UintAuthorityId(44)),
                (1, sp_runtime::testing::UintAuthorityId(1)),
                (10_001, sp_runtime::testing::UintAuthorityId(10_001)),
            ]
        );

        // Rotate session.
        rotate_session();

        // Assert state changes.
        //
        // Expect that the unbanned account has been included to validators list
        // and has been added to queued keys.
        assert!(<BannedAccounts<Test>>::get().is_empty());
        assert_eq!(
            Bioauth::active_authentications(),
            vec![pallet_bioauth::Authentication {
                public_key: 1,
                expires_at: 1000,
            }]
        );
        assert_eq!(Session::validators(), vec![42, 43, 44, 1, 10_001]);
        assert_eq!(
            Session::queued_keys(),
            vec![
                (42, sp_runtime::testing::UintAuthorityId(42)),
                (43, sp_runtime::testing::UintAuthorityId(43)),
                (44, sp_runtime::testing::UintAuthorityId(44)),
                (1, sp_runtime::testing::UintAuthorityId(1)),
                (10_001, sp_runtime::testing::UintAuthorityId(10_001)),
            ]
        );
    });
}
