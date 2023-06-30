use frame_support::{traits::ConstU32, BoundedVec};
use pallet_evm::ExitSucceed;
use precompile_utils::{Bytes, EvmDataWriter};

use crate::{mock::*, *};

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

        // Check test preconditions.
        assert_eq!(Balances::total_balance(&alice), alice_balance);
        assert_eq!(EvmBalances::total_balance(&alice_evm), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Set mock expectations.
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
            Balances::total_balance(&alice),
            alice_balance - swap_balance
        );
        assert_eq!(EvmBalances::total_balance(&alice_evm), swap_balance);
        System::assert_has_event(RuntimeEvent::CurrencySwap(Event::BalancesSwapped {
            from: alice,
            withdrawed_amount: swap_balance,
            to: alice_evm,
            deposited_amount: swap_balance,
        }));

        // Assert mock invocations.
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
        let swap_ctx = MockCurrencySwap::swap_context();
        swap_ctx
            .expect()
            .once()
            .with(predicate::eq(
                <Balances as Currency<u64>>::NegativeImbalance::new(swap_balance),
            ))
            .return_once(move |_| Err(DispatchError::Other("currency swap failed")));

        // Invoke the function under test.
        assert_noop!(
            CurrencySwap::swap(RuntimeOrigin::signed(alice), alice_evm, swap_balance),
            DispatchError::Other("currency swap failed")
        );

        // Assert mock invocations.
        swap_ctx.checkpoint();
    });
}
