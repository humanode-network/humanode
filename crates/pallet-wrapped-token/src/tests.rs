use frame_support::traits::Currency;

use crate::{mock::*, Approvals};

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

/// This test verifies basic approvals related flow.
#[test]
fn approvals_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42_u64;
        let bob = 52_u64;
        let approved_balance = 999;

        // Check test preconditions.
        assert_eq!(<Approvals<Test>>::get(alice, bob), None);

        // Store alice-bob approval.
        <Approvals<Test>>::insert(alice, bob, approved_balance);

        // Verify alice-bob approval existence.
        assert_eq!(<Approvals<Test>>::get(alice, bob), Some(approved_balance));
    })
}
