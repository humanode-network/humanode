//! The tests for the pallet.

use frame_support::{assert_noop, assert_ok, traits::Currency};
use mockall::predicate;
use sp_core::H160;
use sp_runtime::{DispatchError, TokenError};
use sp_std::str::FromStr;

use crate::{mock::*, *};

/// This test verifies that swap call works as expected in case origin left balances amount
/// is greater or equal than existential deposit.
#[test]
fn swap_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42;
        let alice_evm = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let alice_balance = 1000;
        let swap_balance = 100;

        // Prepare the test state.
        Balances::make_free_balance_be(&alice, alice_balance);

        // Check test preconditions.
        assert_eq!(
            <Balances as Currency<_>>::total_balance(&alice),
            alice_balance
        );
        assert_eq!(<EvmBalances as Currency<_>>::total_balance(&alice_evm), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Set mock expectations.
        let estimate_swapped_balance_ctx = MockCurrencySwap::estimate_swapped_balance_context();
        estimate_swapped_balance_ctx
            .expect()
            .once()
            .with(predicate::eq(swap_balance))
            .return_const(swap_balance);
        let swap_ctx = MockCurrencySwap::swap_context();
        swap_ctx
            .expect()
            .once()
            .with(predicate::eq(
                <Balances as Currency<AccountId>>::NegativeImbalance::new(swap_balance),
            ))
            .return_once(move |_| {
                Ok(<EvmBalances as Currency<EvmAccountId>>::NegativeImbalance::new(swap_balance))
            });

        // Invoke the function under test.
        assert_ok!(CurrencySwap::swap(
            RuntimeOrigin::signed(alice),
            alice_evm,
            swap_balance
        ));

        // Assert state changes.
        assert_eq!(
            <Balances as Currency<_>>::total_balance(&alice),
            alice_balance - swap_balance
        );
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            swap_balance
        );
        System::assert_has_event(RuntimeEvent::CurrencySwap(Event::BalancesSwapped {
            from: alice,
            withdrawed_amount: swap_balance,
            to: alice_evm,
            deposited_amount: swap_balance,
        }));

        // Assert mock invocations.
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
    });
}

/// This test verifies that swap call works as expected in case origin left balances amount
/// is less than existential deposit. The origin account should be killed.
#[test]
fn swap_works_kill_origin() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42;
        let alice_evm = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let alice_balance = 1000;
        let swap_balance = 999;

        // Prepare the test state.
        Balances::make_free_balance_be(&alice, alice_balance);

        // Check test preconditions.
        assert_eq!(
            <Balances as Currency<_>>::total_balance(&alice),
            alice_balance
        );
        assert_eq!(<EvmBalances as Currency<_>>::total_balance(&alice_evm), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Set mock expectations.
        let estimate_swapped_balance_ctx = MockCurrencySwap::estimate_swapped_balance_context();
        estimate_swapped_balance_ctx
            .expect()
            .once()
            .with(predicate::eq(swap_balance))
            .return_const(swap_balance);
        let swap_ctx = MockCurrencySwap::swap_context();
        swap_ctx
            .expect()
            .once()
            .with(predicate::eq(
                <Balances as Currency<AccountId>>::NegativeImbalance::new(swap_balance),
            ))
            .return_once(move |_| {
                Ok(<EvmBalances as Currency<EvmAccountId>>::NegativeImbalance::new(swap_balance))
            });

        // Invoke the function under test.
        assert_ok!(CurrencySwap::swap(
            RuntimeOrigin::signed(alice),
            alice_evm,
            swap_balance
        ));

        // Assert state changes.
        assert!(!System::account_exists(&alice));
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            swap_balance
        );
        System::assert_has_event(RuntimeEvent::CurrencySwap(Event::BalancesSwapped {
            from: alice,
            withdrawed_amount: swap_balance,
            to: alice_evm,
            deposited_amount: swap_balance,
        }));

        // Assert mock invocations.
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
    });
}

/// This test verifies that `swap_keep_alive` call works in the happy path.
#[test]
fn swap_keep_alive_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42;
        let alice_evm = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let alice_balance = 1000;
        let swap_balance = 100;

        // Prepare the test state.
        Balances::make_free_balance_be(&alice, alice_balance);

        // Check test preconditions.
        assert_eq!(
            <Balances as Currency<_>>::total_balance(&alice),
            alice_balance
        );
        assert_eq!(<EvmBalances as Currency<_>>::total_balance(&alice_evm), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Set mock expectations.
        let estimate_swapped_balance_ctx = MockCurrencySwap::estimate_swapped_balance_context();
        estimate_swapped_balance_ctx
            .expect()
            .once()
            .with(predicate::eq(swap_balance))
            .return_const(swap_balance);
        let swap_ctx = MockCurrencySwap::swap_context();
        swap_ctx
            .expect()
            .once()
            .with(predicate::eq(
                <Balances as Currency<AccountId>>::NegativeImbalance::new(swap_balance),
            ))
            .return_once(move |_| {
                Ok(<EvmBalances as Currency<EvmAccountId>>::NegativeImbalance::new(swap_balance))
            });

        // Invoke the function under test.
        assert_ok!(CurrencySwap::swap(
            RuntimeOrigin::signed(alice),
            alice_evm,
            swap_balance
        ));

        // Assert state changes.
        assert_eq!(
            <Balances as Currency<_>>::total_balance(&alice),
            alice_balance - swap_balance
        );
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            swap_balance
        );
        System::assert_has_event(RuntimeEvent::CurrencySwap(Event::BalancesSwapped {
            from: alice,
            withdrawed_amount: swap_balance,
            to: alice_evm,
            deposited_amount: swap_balance,
        }));

        // Assert mock invocations.
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
    });
}

/// This test verifies that swap call fails in case some error happens during the actual swap logic.
#[test]
fn swap_fails() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42;
        let alice_evm = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let alice_balance = 1000;
        let swap_balance = 100;

        // Prepare the test state.
        Balances::make_free_balance_be(&alice, alice_balance);

        // Set mock expectations.
        let estimate_swapped_balance_ctx = MockCurrencySwap::estimate_swapped_balance_context();
        estimate_swapped_balance_ctx
            .expect()
            .once()
            .with(predicate::eq(swap_balance))
            .return_const(swap_balance);
        let swap_ctx = MockCurrencySwap::swap_context();
        swap_ctx
            .expect()
            .once()
            .with(predicate::eq(
                <Balances as Currency<u64>>::NegativeImbalance::new(swap_balance),
            ))
            .return_once(move |incoming_imbalance| {
                Err(primitives_currency_swap::Error {
                    cause: sp_runtime::DispatchError::Other("currency swap failed"),
                    incoming_imbalance,
                })
            });

        // Invoke the function under test.
        assert_noop!(
            CurrencySwap::swap(RuntimeOrigin::signed(alice), alice_evm, swap_balance),
            DispatchError::Other("currency swap failed")
        );

        // Assert state changes.
        assert_eq!(
            <Balances as Currency<_>>::total_balance(&alice),
            alice_balance
        );
        assert_eq!(<EvmBalances as Currency<_>>::total_balance(&alice_evm), 0);

        // Assert mock invocations.
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
    });
}

/// This test verifies that swap call fails in case estimated swapped balance less or equal
/// than target currency existential deposit.
#[test]
fn swap_below_ed_fails() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42;
        let alice_evm = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let alice_balance = 1000;
        let swap_balance = 100;

        // Prepare the test state.
        Balances::make_free_balance_be(&alice, alice_balance);

        // // Set mock expectations.
        let estimate_swapped_balance_ctx = MockCurrencySwap::estimate_swapped_balance_context();
        estimate_swapped_balance_ctx
            .expect()
            .once()
            .with(predicate::eq(swap_balance))
            .return_const(EXISTENTIAL_DEPOSIT - 1);
        let swap_ctx = MockCurrencySwap::swap_context();
        swap_ctx.expect().never();

        // Invoke the function under test.
        assert_noop!(
            CurrencySwap::swap(RuntimeOrigin::signed(alice), alice_evm, swap_balance),
            DispatchError::Token(TokenError::BelowMinimum)
        );

        // Assert state changes.
        assert_eq!(
            <Balances as Currency<_>>::total_balance(&alice),
            alice_balance
        );
        assert_eq!(<EvmBalances as Currency<_>>::total_balance(&alice_evm), 0);

        // Assert mock invocations.
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
    });
}

/// This test verifies that `swap_keep_alive` call fails in case origin left balances amount
/// is less than existential deposit. The call should prevent swap operation.
#[test]
fn swap_keep_alive_fails() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42;
        let alice_evm = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let alice_balance = 1000;
        let swap_balance = 999;

        // Prepare the test state.
        Balances::make_free_balance_be(&alice, alice_balance);

        // Set mock expectations.
        let estimate_swapped_balance_ctx = MockCurrencySwap::estimate_swapped_balance_context();
        estimate_swapped_balance_ctx
            .expect()
            .once()
            .with(predicate::eq(swap_balance))
            .return_const(swap_balance);
        let swap_ctx = MockCurrencySwap::swap_context();
        swap_ctx.expect().never();

        // Invoke the function under test.
        assert_noop!(
            CurrencySwap::swap_keep_alive(RuntimeOrigin::signed(alice), alice_evm, swap_balance),
            pallet_balances::Error::<Test>::Expendability
        );

        // Assert state changes.
        assert_eq!(
            <Balances as Currency<_>>::total_balance(&alice),
            alice_balance
        );
        assert_eq!(<EvmBalances as Currency<_>>::total_balance(&alice_evm), 0);

        // Assert mock invocations.
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
    });
}

/// This test verifies that `swap_keep_alive` call fails in case estimated swapped balance less or equal
/// than target currency existential deposit.
#[test]
fn swap_keep_alive_below_ed_fails() {
    new_test_ext().execute_with_ext(|_| {
        let alice = 42;
        let alice_evm = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let alice_balance = 1000;
        let swap_balance = 100;

        // Prepare the test state.
        Balances::make_free_balance_be(&alice, alice_balance);

        // // Set mock expectations.
        let estimate_swapped_balance_ctx = MockCurrencySwap::estimate_swapped_balance_context();
        estimate_swapped_balance_ctx
            .expect()
            .once()
            .with(predicate::eq(swap_balance))
            .return_const(EXISTENTIAL_DEPOSIT - 1);
        let swap_ctx = MockCurrencySwap::swap_context();
        swap_ctx.expect().never();

        // Invoke the function under test.
        assert_noop!(
            CurrencySwap::swap_keep_alive(RuntimeOrigin::signed(alice), alice_evm, swap_balance),
            DispatchError::Token(TokenError::BelowMinimum)
        );

        // Assert state changes.
        assert_eq!(
            <Balances as Currency<_>>::total_balance(&alice),
            alice_balance
        );
        assert_eq!(<EvmBalances as Currency<_>>::total_balance(&alice_evm), 0);

        // Assert mock invocations.
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
    });
}
