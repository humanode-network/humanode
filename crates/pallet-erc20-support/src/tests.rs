use frame_support::{assert_noop, assert_ok, sp_runtime::TokenError, traits::Currency};
use sp_core::U256;

use crate::{mock::*, *};

/// This test verifies that getting total supply works as expected.
#[test]
fn total_supply_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42;
        let alice_balance = 1000;
        let bob = 43;
        let bob_balance = 2000;

        // Prepare the test state.
        Balances::make_free_balance_be(&alice, alice_balance);
        Balances::make_free_balance_be(&bob, bob_balance);

        // Check total supply.
        assert_eq!(Erc20Balances::total_supply(), alice_balance + bob_balance);
    });
}

/// This test verifies that getting balance of provided account works as expected.
#[test]
fn balance_of_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42;
        let alice_balance = 1000;

        // Prepare the test state.
        Balances::make_free_balance_be(&alice, alice_balance);

        // Check Alice's balance value.
        assert_eq!(Erc20Balances::balance_of(&alice), alice_balance);
    });
}

/// This test verifies that approval logic works as expected.
#[test]
fn approve_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42_u64;
        let bob = 52_u64;
        let approved_balance = 999;

        // Check test preconditions.
        assert_eq!(Erc20Balances::approvals(alice, bob), 0.into());

        // Store alice-bob approval.
        Erc20Balances::approve(alice, bob, approved_balance.into());

        // Verify alice-bob approval existence.
        assert_eq!(
            Erc20Balances::approvals(alice, bob),
            approved_balance.into()
        );
    })
}

/// This test verifies that approval logic works as expected in case approving `U256::MAX-1` value.
#[test]
fn approve_max_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42_u64;
        let bob = 52_u64;
        let approved_balance = U256::MAX - 1;

        // Check test preconditions.
        assert_eq!(Erc20Balances::approvals(alice, bob), 0.into());

        // Store alice-bob approval.
        Erc20Balances::approve(alice, bob, approved_balance);

        // Verify alice-bob approval existence.
        assert_eq!(Erc20Balances::approvals(alice, bob), approved_balance);
    })
}

/// This test verifies that approval logic works as expected in case approval has been overwritten.
#[test]
fn approve_overwrite_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42_u64;
        let bob = 52_u64;
        let approved_balance = 999;
        let approved_balance_new = 1000;

        // Check test preconditions.
        assert_eq!(Erc20Balances::approvals(alice, bob), 0.into());

        // Alice approves balance value for Bob.
        Erc20Balances::approve(alice, bob, approved_balance.into());
        // Verify alice-bob approval existence.
        assert_eq!(
            Erc20Balances::approvals(alice, bob),
            approved_balance.into()
        );

        // Alice approves new balance value for Bob.
        Erc20Balances::approve(alice, bob, approved_balance_new.into());
        // Verify alice-bob approval existence.
        assert_eq!(
            Erc20Balances::approvals(alice, bob),
            approved_balance_new.into()
        );
    })
}

/// This test verifies that approval logic works as expected in case approved balanced
/// has been transferred in single transaction.
#[test]
fn approve_spend_all_in_single_transaction_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42_u64;
        let bob = 52_u64;
        let charlie = 62_u64;
        let alice_balance = 10000;
        let approved_balance = 1000;

        // Prepare the test state.
        Balances::make_free_balance_be(&alice, alice_balance);
        Erc20Balances::approve(alice, bob, approved_balance.into());

        // Check test preconditions.
        assert_eq!(
            Erc20Balances::approvals(&alice, &bob),
            approved_balance.into()
        );

        // Execute transfer_from.
        assert_ok!(Erc20Balances::transfer_from(
            bob,
            alice,
            charlie,
            approved_balance
        ));

        // Check resulted approvals.
        assert_eq!(Erc20Balances::approvals(&alice, &bob), 0.into());
        // Check resulted balances.
        assert_eq!(
            Balances::total_balance(&alice),
            alice_balance - approved_balance
        );
        assert_eq!(Balances::total_balance(&bob), 0);
        assert_eq!(Balances::total_balance(&charlie), approved_balance);
        // Check transfer_from failed execution.
        assert_noop!(
            Erc20Balances::transfer_from(bob, alice, charlie, 1),
            <Error<Test>>::SpendMoreThanAllowed
        );
    })
}

/// This test verifies that approval logic works as expected in case approved balanced
/// has been transferred in several transactions.
#[test]
fn approve_spend_all_in_several_transactions_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42_u64;
        let bob = 52_u64;
        let charlie = 62_u64;
        let alice_balance = 10000;
        let approved_balance = 1000;

        // Prepare the test state.
        Balances::make_free_balance_be(&alice, alice_balance);
        Erc20Balances::approve(alice, bob, approved_balance.into());

        // Check test preconditions.
        assert_eq!(
            Erc20Balances::approvals(&alice, &bob),
            approved_balance.into()
        );

        // Execute transfer_from.
        assert_ok!(Erc20Balances::transfer_from(bob, alice, charlie, 500));

        // Check resulted approvals.
        assert_eq!(
            Erc20Balances::approvals(&alice, &bob),
            (approved_balance - 500).into()
        );
        // Check resulted balances.
        assert_eq!(Balances::total_balance(&alice), alice_balance - 500);
        assert_eq!(Balances::total_balance(&bob), 0);
        assert_eq!(Balances::total_balance(&charlie), 500);

        // Execute transfer_from again.
        assert_ok!(Erc20Balances::transfer_from(
            bob,
            alice,
            charlie,
            approved_balance - 500
        ));

        // Check resulted approvals.
        assert_eq!(Erc20Balances::approvals(&alice, &bob), 0.into());
        // Check resulted balances.
        assert_eq!(
            Balances::total_balance(&alice),
            alice_balance - approved_balance
        );
        assert_eq!(Balances::total_balance(&bob), 0);
        assert_eq!(Balances::total_balance(&charlie), approved_balance);
        // Check transfer_from failed execution.
        assert_noop!(
            Erc20Balances::transfer_from(bob, alice, charlie, 1),
            <Error<Test>>::SpendMoreThanAllowed
        );
    })
}

/// This test verifies that approval logic works as expected in case approved balance value
/// is more than owner's balance value initially.
#[test]
fn approve_approval_value_more_than_balance_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42_u64;
        let alice_stash = 43_u64;
        let bob = 52_u64;
        let charlie = 62_u64;
        let alice_balance_initial = 1000;
        let alice_stash_balance = 1000;
        let approved_balance = 2000;

        // Prepare the test state.
        Balances::make_free_balance_be(&alice, alice_balance_initial);
        Balances::make_free_balance_be(&alice_stash, alice_stash_balance);
        Erc20Balances::approve(alice, bob, approved_balance.into());

        // Check test preconditions.
        assert_eq!(
            Erc20Balances::approvals(&alice, &bob),
            approved_balance.into()
        );

        // Try to execute transfer_from with all approved balance.
        assert_noop!(
            Erc20Balances::transfer_from(bob, alice, charlie, approved_balance),
            TokenError::FundsUnavailable
        );

        // Execute transfer_from with alice initial balance value.
        assert_ok!(Erc20Balances::transfer_from(
            bob,
            alice,
            charlie,
            alice_balance_initial
        ));

        // Check resulted approvals.
        assert_eq!(
            Erc20Balances::approvals(&alice, &bob),
            (approved_balance - alice_balance_initial).into()
        );
        // Check resulted balances.
        assert_eq!(Balances::total_balance(&alice), 0);
        assert_eq!(Balances::total_balance(&bob), 0);
        assert_eq!(Balances::total_balance(&charlie), alice_balance_initial);

        // Send more tokens to alice.
        assert_ok!(Erc20Balances::transfer(
            alice_stash,
            alice,
            alice_stash_balance
        ));

        // Execute transfer_from with the rest approved balance value.
        assert_ok!(Erc20Balances::transfer_from(
            bob,
            alice,
            charlie,
            approved_balance - alice_balance_initial
        ));

        // Check resulted approvals.
        assert_eq!(Erc20Balances::approvals(&alice, &bob), 0.into());
        // Check transfer_from failed execution.
        assert_noop!(
            Erc20Balances::transfer_from(bob, alice, charlie, 1),
            <Error<Test>>::SpendMoreThanAllowed
        );
    })
}

/// This test verifies that transferring logic works as expected.
#[test]
fn transfer_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42;
        let alice_balance = 10000;
        let bob = 43;
        let transferred_balance = 5000;

        // Prepare the test state.
        Balances::make_free_balance_be(&alice, alice_balance);

        // Execute transfer.
        assert_ok!(Erc20Balances::transfer(alice, bob, transferred_balance));

        // Check resulted balances.
        assert_eq!(
            Balances::total_balance(&alice),
            alice_balance - transferred_balance
        );
        assert_eq!(Balances::total_balance(&bob), transferred_balance);
    });
}

/// This test verifies that transferring logic on behalf of provided account works as expected.
#[test]
fn transfer_from_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42;
        let alice_balance = 10000;
        let bob = 43;
        let approved_balance = 5000;
        let charlie = 44;
        let transferred_from_balance = 1000;

        // Prepare the test state.
        Balances::make_free_balance_be(&alice, alice_balance);
        Erc20Balances::approve(alice, bob, approved_balance.into());

        // Execute transfer_from.
        assert_ok!(Erc20Balances::transfer_from(
            bob,
            alice,
            charlie,
            transferred_from_balance
        ));

        // Check resulted balances.
        assert_eq!(
            Balances::total_balance(&alice),
            alice_balance - transferred_from_balance
        );
        assert_eq!(Balances::total_balance(&bob), 0);
        assert_eq!(Balances::total_balance(&charlie), transferred_from_balance);

        // Check updated approvals changes.
        assert_eq!(
            Erc20Balances::approvals(alice, bob),
            (approved_balance - transferred_from_balance).into()
        );
    });
}

/// This test verifies that transferring logic on behalf of provided account fails in case
/// the corresponding approval doesn't have sufficient balance.
#[test]
fn transfer_from_fails_spend_more_than_allowed() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42;
        let alice_balance = 10000;
        let bob = 43;
        let approved_balance = 500;
        let charlie = 44;
        let transferred_from_balance = 1000;

        // Prepare the test state.
        Balances::make_free_balance_be(&alice, alice_balance);
        Erc20Balances::approve(alice, bob, approved_balance.into());

        // Execute transfer_from.
        assert_noop!(
            Erc20Balances::transfer_from(bob, alice, charlie, transferred_from_balance),
            <Error<Test>>::SpendMoreThanAllowed
        );
    });
}
