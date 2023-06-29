//! The tests for the pallet.

use frame_support::{assert_noop, assert_ok, traits::Currency};
use sp_core::H160;
use sp_std::str::FromStr;

use crate::mock::*;

/// This test verifies that swap call works in the happy path.
#[test]
fn swap_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42;
        let alice_evm = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let alice_balance = 1000;
        let swap_balance = 100;

        // Prepare the test state.
        Balances::make_free_balance_be(&alice, alice_balance);

        let balances_before = Balances::total_issuance();
        let evm_balances_before = EvmBalances::total_issuance();

        // Check test preconditions.
        assert_eq!(Balances::total_balance(&alice), alice_balance);
        assert_eq!(EvmBalances::total_balance(&alice_evm), 0);

        // Invoke the function under test.
        assert_ok!(CurrencySwap::swap(
            RuntimeOrigin::signed(alice),
            alice_evm,
            swap_balance
        ));

        // Assert state changes.
        assert_eq!(
            Balances::total_balance(&alice),
            alice_balance - swap_balance
        );
        assert_eq!(EvmBalances::total_balance(&alice_evm), swap_balance);
        assert_eq!(Balances::total_issuance(), balances_before - swap_balance);
        assert_eq!(
            EvmBalances::total_issuance(),
            evm_balances_before + swap_balance
        );
    });
}

/// This test verifies that swap call fails in case some error happens during the actual swap logic.
#[test]
fn swap_fails() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42;
        let alice_evm = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let alice_balance = 1000;
        let swap_balance = 10000;

        // Prepare the test state.
        Balances::make_free_balance_be(&alice, alice_balance);

        // Invoke the function under test.
        assert_noop!(
            CurrencySwap::swap(RuntimeOrigin::signed(alice), alice_evm, swap_balance),
            pallet_balances::Error::<Test>::InsufficientBalance
        );
    });
}
