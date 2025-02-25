use fp_evm::{ExitReason, ExitSucceed};
use frame_support::{
    assert_noop, assert_ok,
    dispatch::DispatchError,
    sp_runtime::{ArithmeticError, TokenError},
    traits::fungible::Unbalanced,
};
use sp_core::Get;

use crate::{
    mock::{alice as source_swap_native_account, *},
    *,
};

/// Returns target swap evm account used in tests.
fn target_swap_evm_account() -> EvmAccountId {
    EvmAccountId::from(hex_literal::hex!(
        "7700000000000000000000000000000000000077"
    ))
}

/// A helper to identify call used in tests.
enum TestCall {
    Swap,
    SwapKeepAlive,
}

/// A helper function to run succeeded test and assert state changes.
fn run_succeeded_test_and_assert(
    call: TestCall,
    swap_balance: Balance,
    expected_left_origin_balance: Balance,
) {
    // Check test preconditions.
    assert_eq!(
        Balances::total_balance(&source_swap_native_account()),
        INIT_BALANCE
    );
    assert_eq!(EvmBalances::total_balance(&target_swap_evm_account()), 0);

    // We should remember expected evm transaction hash before execution as nonce is increased
    // after the execution.
    let expected_evm_transaction_hash = ethereum_transfer_transaction::<Test>(
        BridgePotEvm::get(),
        target_swap_evm_account(),
        swap_balance,
    )
    .hash();

    // Set block number to enable events.
    System::set_block_number(1);

    // Invoke the function under test.
    assert_ok!(match call {
        TestCall::Swap => EvmSwap::swap(
            RuntimeOrigin::signed(source_swap_native_account()),
            target_swap_evm_account(),
            swap_balance
        ),
        TestCall::SwapKeepAlive => EvmSwap::swap_keep_alive(
            RuntimeOrigin::signed(source_swap_native_account()),
            target_swap_evm_account(),
            swap_balance
        ),
    });

    // Assert state changes.

    // Verify that source swap native balance is equal to expected left origin balance value.
    assert_eq!(
        <Balances>::total_balance(&source_swap_native_account()),
        expected_left_origin_balance,
    );
    // Verify that bridge pot native balance has been increased by swap value.
    assert_eq!(
        Balances::total_balance(&BridgePotNative::get()),
        BRIDGE_INIT_BALANCE + swap_balance,
    );
    // Verify that target swap evm balance has been increased by swap value.
    assert_eq!(
        <EvmBalances>::total_balance(&target_swap_evm_account()),
        swap_balance
    );
    // Verify that bridge pot evm balance has been decreased by swap value.
    assert_eq!(
        EvmBalances::total_balance(&BridgePotEvm::get()),
        BRIDGE_INIT_BALANCE - swap_balance,
    );
    // Verify that precompile balance remains the same.
    assert_eq!(EvmBalances::total_balance(&*PRECOMPILE_ADDRESS), 0);
    // Verifyt that we have a corresponding evm swap event.
    System::assert_has_event(RuntimeEvent::EvmSwap(Event::BalancesSwapped {
        from: source_swap_native_account(),
        withdrawed_amount: swap_balance,
        to: target_swap_evm_account(),
        deposited_amount: swap_balance,
        evm_transaction_hash: expected_evm_transaction_hash,
    }));
    // Verify that we have a corresponding ethereum event.
    System::assert_has_event(RuntimeEvent::Ethereum(pallet_ethereum::Event::Executed {
        from: BridgePotEvm::get(),
        to: target_swap_evm_account(),
        transaction_hash: expected_evm_transaction_hash,
        exit_reason: ExitReason::Succeed(ExitSucceed::Stopped),
        extra_data: vec![],
    }));
}

/// This test verifies that the `swap` call works in the happy path.
#[test]
fn swap_works() {
    new_test_ext().execute_with_ext(|_| {
        run_succeeded_test_and_assert(TestCall::Swap, 100, INIT_BALANCE - 100);
    });
}

/// This test verifies that `swap` call works as expected in case origin left balances amount
/// is less than existential deposit. The origin account should be killed.
#[test]
fn swap_works_kill_origin() {
    new_test_ext().execute_with_ext(|_| {
        run_succeeded_test_and_assert(TestCall::Swap, INIT_BALANCE - 1, 0);
    });
}

/// This test verifies that `swap_keep_alive` call works in the happy path.
#[test]
fn swap_keep_alive_works() {
    new_test_ext().execute_with_ext(|_| {
        run_succeeded_test_and_assert(TestCall::SwapKeepAlive, 100, INIT_BALANCE - 100);
    });
}

/// This test verifies that `swap` call fails in case source account has no the sufficient balance.
#[test]
fn swap_fails_no_funds() {
    new_test_ext().execute_with_ext(|_| {
        let swap_balance = INIT_BALANCE + 1;

        // Invoke the function under test.
        assert_noop!(
            EvmSwap::swap(
                RuntimeOrigin::signed(source_swap_native_account()),
                target_swap_evm_account(),
                swap_balance
            ),
            DispatchError::Token(TokenError::FundsUnavailable)
        );
    });
}

/// This test verifies that `swap` call fails in case target deposit results into overflow.
#[test]
fn swap_fails_overflow() {
    new_test_ext().execute_with_ext(|_| {
        let swap_balance = 1;

        EvmBalances::write_balance(&target_swap_evm_account(), Balance::MAX).unwrap();

        // Invoke the function under test.
        assert_noop!(
            EvmSwap::swap(
                RuntimeOrigin::signed(source_swap_native_account()),
                target_swap_evm_account(),
                swap_balance
            ),
            DispatchError::Arithmetic(ArithmeticError::Overflow)
        );
    });
}

/// This test verifies that `swap_keep_alive` call fails in case origin left balances amount
/// is less than existential deposit. The call should prevent swap operation.
#[test]
fn swap_keep_alive_fails() {
    new_test_ext().execute_with_ext(|_| {
        let swap_balance = INIT_BALANCE - 1;

        // Invoke the function under test.
        assert_noop!(
            EvmSwap::swap_keep_alive(
                RuntimeOrigin::signed(source_swap_native_account()),
                target_swap_evm_account(),
                swap_balance
            ),
            DispatchError::Token(TokenError::NotExpendable)
        );
    });
}

/// This test verifies that `swap_keep_alive` call fails in case source account has no the sufficient balance.
#[test]
fn swap_keep_alive_fails_no_funds() {
    new_test_ext().execute_with_ext(|_| {
        let swap_balance = INIT_BALANCE + 1;

        // Invoke the function under test.
        assert_noop!(
            EvmSwap::swap_keep_alive(
                RuntimeOrigin::signed(source_swap_native_account()),
                target_swap_evm_account(),
                swap_balance
            ),
            DispatchError::Token(TokenError::FundsUnavailable)
        );
    });
}

/// This test verifies that `swap_keep_alive` call fails in case target deposit results into overflow.
#[test]
fn swap_keep_alive_fails_overflow() {
    new_test_ext().execute_with_ext(|_| {
        let swap_balance = 1;

        EvmBalances::write_balance(&target_swap_evm_account(), Balance::MAX).unwrap();

        // Invoke the function under test.
        assert_noop!(
            EvmSwap::swap_keep_alive(
                RuntimeOrigin::signed(source_swap_native_account()),
                target_swap_evm_account(),
                swap_balance
            ),
            DispatchError::Arithmetic(ArithmeticError::Overflow)
        );
    });
}
