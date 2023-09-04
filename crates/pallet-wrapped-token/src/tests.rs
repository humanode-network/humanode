use frame_support::{assert_ok, traits::Currency};

use crate::mock::*;

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
        assert_eq!(WrappedBalances::total_supply(), alice_balance + bob_balance);
    });
}

#[test]
fn balance_of_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42;
        let alice_balance = 1000;

        // Prepare the test state.
        Balances::make_free_balance_be(&alice, alice_balance);

        // Check Alice's balance value.
        assert_eq!(WrappedBalances::balance_of(&alice), alice_balance);
    });
}

#[test]
fn approve_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42_u64;
        let bob = 52_u64;
        let approved_balance = 999;

        // Check test preconditions.
        assert_eq!(WrappedBalances::approvals(alice, bob), None);

        // Store alice-bob approval.
        WrappedBalances::approve(alice, bob, approved_balance);

        // Verify alice-bob approval existence.
        assert_eq!(
            WrappedBalances::approvals(alice, bob),
            Some(approved_balance)
        );
    })
}

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
        assert_ok!(WrappedBalances::transfer(alice, bob, transferred_balance));

        // Check resulted balances.
        assert_eq!(
            Balances::total_balance(&alice),
            alice_balance - transferred_balance
        );
        assert_eq!(Balances::total_balance(&bob), transferred_balance);
    });
}

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
        WrappedBalances::approve(alice, bob, approved_balance);

        // Execute transfer_from.
        assert_ok!(WrappedBalances::transfer_from(
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
            WrappedBalances::approvals(alice, bob),
            Some(approved_balance - transferred_from_balance)
        );
    });
}
