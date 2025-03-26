// Allow simple integer arithmetic in tests.
#![allow(clippy::arithmetic_side_effects)]

use fp_evm::{ExitError, ExitReason};
use frame_support::{assert_noop, traits::fungible::Unbalanced};
use pallet_evm::Runner;
use precompile_utils::{EvmDataWriter, LogsBuilder};
use sp_core::H256;

use crate::{mock::*, *};

/// Returns source swap evm account used in tests.
fn source_swap_evm_account() -> EvmAccountId {
    EvmAccountId::from(hex_literal::hex!(
        "1100000000000000000000000000000000000011"
    ))
}

/// Returns target swap native account used in tests.
fn target_swap_native_account() -> AccountId {
    AccountId::from(hex_literal::hex!(
        "7700000000000000000000000000000000000000000000000000000000000077"
    ))
}

/// A utility that performs gas to fee computation.
fn gas_to_fee(gas: u64) -> Balance {
    Balance::from(gas) * Balance::try_from(*GAS_PRICE).unwrap()
}

/// A helper function to run succeeded test and assert state changes.
fn run_succeeded_test_and_assert(
    swap_balance: Balance,
    expected_gas_usage: u64,
    expected_fee: Balance,
) {
    let source_swap_evm_account_balance_before =
        EvmBalances::total_balance(&source_swap_evm_account());
    let bridge_pot_evm_account_balance_before = EvmBalances::total_balance(&BridgePotEvm::get());
    let bridge_pot_native_account_balance_before = Balances::total_balance(&BridgePotNative::get());
    let target_swap_native_account_balance_before =
        Balances::total_balance(&target_swap_native_account());

    // Invoke the function under test.
    let execinfo = <Test as pallet_evm::Config>::Runner::call(
        source_swap_evm_account(),
        *PRECOMPILE_ADDRESS,
        EvmDataWriter::new_with_selector(Action::Swap)
            .write(H256::from(target_swap_native_account().as_ref()))
            .build(),
        swap_balance.into(),
        expected_gas_usage, // the exact amount of fee we'll be using
        Some(*GAS_PRICE),
        Some(*GAS_PRICE),
        None,
        Vec::new(),
        true,
        true,
        None,
        None,
        <Test as pallet_evm::Config>::config(),
    )
    .unwrap();
    assert_eq!(
        execinfo.exit_reason,
        fp_evm::ExitReason::Succeed(fp_evm::ExitSucceed::Returned)
    );
    assert_eq!(execinfo.used_gas.standard, expected_gas_usage.into());
    assert_eq!(execinfo.value, EvmDataWriter::new().write(true).build());
    assert_eq!(
        execinfo.logs,
        vec![LogsBuilder::new(*PRECOMPILE_ADDRESS).log3(
            SELECTOR_LOG_SWAP,
            source_swap_evm_account(),
            H256::from(target_swap_native_account().as_ref()),
            EvmDataWriter::new().write(swap_balance).build(),
        )]
    );

    // Assert state changes.

    // Verify that source swap evm balance has been decreased by swap value and fee.
    assert_eq!(
        <EvmBalances>::total_balance(&source_swap_evm_account()),
        source_swap_evm_account_balance_before - swap_balance - expected_fee,
    );
    // Verify that bridge pot evm balance has been increased by swap value.
    assert_eq!(
        EvmBalances::total_balance(&BridgePotEvm::get()),
        bridge_pot_evm_account_balance_before + swap_balance,
    );
    // Verify that target swap native balance has been increased by swap value.
    assert_eq!(
        <Balances>::total_balance(&target_swap_native_account()),
        target_swap_native_account_balance_before + swap_balance
    );
    // Verify that bridge pot native balance has been decreased by swap value.
    assert_eq!(
        Balances::total_balance(&BridgePotNative::get()),
        bridge_pot_native_account_balance_before - swap_balance,
    );
    // Verify that precompile balance remains the same.
    assert_eq!(EvmBalances::total_balance(&*PRECOMPILE_ADDRESS), 0);
}

/// This test verifies that the swap precompile call works in the happy path.
#[test]
fn swap_works() {
    new_test_ext().execute_with_ext(|_| {
        let expected_gas_usage: u64 = 21216 + 200;
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        run_succeeded_test_and_assert(100, expected_gas_usage, expected_fee);
    });
}

/// This test verifies that the swap precompile call works when we transfer *almost* the full
/// account balance.
/// Almost because we leave one token left on the source account.
#[test]
fn swap_works_almost_full_balance() {
    new_test_ext().execute_with_ext(|_| {
        let expected_gas_usage: u64 = 21216 + 200;
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        run_succeeded_test_and_assert(
            INIT_BALANCE - expected_fee - 1,
            expected_gas_usage,
            expected_fee,
        );
    });
}

/// This test verifies that the swap precompile call works when we transfer the full account balance.
#[test]
fn swap_works_full_balance() {
    new_test_ext().execute_with_ext(|_| {
        let expected_gas_usage: u64 = 21216 + 200;
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        run_succeeded_test_and_assert(
            INIT_BALANCE - expected_fee,
            expected_gas_usage,
            expected_fee,
        );
    });
}

/// A helper function to run failed test and assert state changes.
fn run_failed_test_and_assert(
    input: Vec<u8>,
    value: U256,
    expected_gas_usage: u64,
    expected_fee: Balance,
    expected_exit_reason: fp_evm::ExitReason,
    expected_exit_value: Vec<u8>,
) {
    let source_swap_evm_account_balance_before =
        EvmBalances::total_balance(&source_swap_evm_account());
    let bridge_pot_evm_account_balance_before = EvmBalances::total_balance(&BridgePotEvm::get());
    let bridge_pot_native_account_balance_before = Balances::total_balance(&BridgePotNative::get());
    let target_swap_native_account_balance_before =
        Balances::total_balance(&target_swap_native_account());

    // Invoke the function under test.
    let execinfo = <Test as pallet_evm::Config>::Runner::call(
        source_swap_evm_account(),
        *PRECOMPILE_ADDRESS,
        input,
        value,
        50_000, // a reasonable upper bound for tests
        Some(*GAS_PRICE),
        Some(*GAS_PRICE),
        None,
        Vec::new(),
        true,
        true,
        None,
        None,
        <Test as pallet_evm::Config>::config(),
    )
    .unwrap();
    assert_eq!(execinfo.exit_reason, expected_exit_reason);
    assert_eq!(execinfo.used_gas.standard, expected_gas_usage.into());
    assert_eq!(execinfo.value, expected_exit_value);
    assert_eq!(execinfo.logs, vec![]);

    // Verify that source swap evm balance is reduced just by spent fee.
    assert_eq!(
        <EvmBalances>::total_balance(&source_swap_evm_account()),
        source_swap_evm_account_balance_before - expected_fee,
    );
    // Verify that bridge pot evm balance remains the same.
    assert_eq!(
        EvmBalances::total_balance(&BridgePotEvm::get()),
        bridge_pot_evm_account_balance_before,
    );
    // Verify that target swap native balance remains the same.
    assert_eq!(
        <Balances>::total_balance(&target_swap_native_account()),
        target_swap_native_account_balance_before
    );
    // Verify that bridge pot native balance remains the same.
    assert_eq!(
        Balances::total_balance(&BridgePotNative::get()),
        bridge_pot_native_account_balance_before,
    );
    // Verify that precompile balance remains the same.
    assert_eq!(EvmBalances::total_balance(&*PRECOMPILE_ADDRESS), 0);
}

/// This test verifies that the swap precompile call fails when estimated swapped balance is
/// less or equal than native token existential deposit.
#[test]
fn swap_fail_target_balance_below_ed() {
    new_test_ext().execute_with_ext(|_| {
        let expected_gas_usage: u64 = 50_000; // all passed gas will be consumed
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        run_failed_test_and_assert(
            EvmDataWriter::new_with_selector(Action::Swap)
                .write(H256::from(target_swap_native_account().as_ref()))
                .build(),
            U256::from(1),
            expected_gas_usage,
            expected_fee,
            ExitReason::Error(ExitError::Other(
                "resulted balance is less than existential deposit".into(),
            )),
            EvmDataWriter::new().build(),
        );
    });
}

/// This test verifies that the swap precompile call fails when estimated swapped balance results
/// into target swap native account balance overflow.
#[test]
fn swap_fail_target_balance_overflow() {
    new_test_ext().execute_with_ext(|_| {
        let expected_gas_usage: u64 = 50_000; // all passed gas will be consumed
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        Balances::write_balance(&target_swap_native_account(), Balance::MAX).unwrap();

        run_failed_test_and_assert(
            EvmDataWriter::new_with_selector(Action::Swap)
                .write(H256::from(target_swap_native_account().as_ref()))
                .build(),
            U256::from(1),
            expected_gas_usage,
            expected_fee,
            ExitReason::Error(ExitError::Other("unable to execute swap".into())),
            EvmDataWriter::new().build(),
        );
    });
}

/// This test verifies that the swap precompile call fails when swapped balance results into
/// bridge pot evm account balance overflow.
#[test]
fn swap_fail_bridge_evm_overflow() {
    new_test_ext().execute_with_ext(|_| {
        let expected_gas_usage: u64 = 50_000; // all passed gas will be consumed
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        EvmBalances::write_balance(&BridgePotEvm::get(), Balance::MAX).unwrap();

        run_failed_test_and_assert(
            EvmDataWriter::new_with_selector(Action::Swap)
                .write(H256::from(target_swap_native_account().as_ref()))
                .build(),
            U256::from(100),
            expected_gas_usage,
            expected_fee,
            ExitReason::Error(ExitError::Other("unable to execute swap".into())),
            EvmDataWriter::new().build(),
        );
    });
}

/// This test verifies that the swap precompile call fails when swap results into killing bridge
/// pot native account.
#[test]
fn swap_fail_bridge_native_killed() {
    new_test_ext().execute_with_ext(|_| {
        let expected_gas_usage: u64 = 50_000; // all passed gas will be consumed
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        EvmBalances::write_balance(
            &source_swap_evm_account(),
            INIT_BALANCE + BRIDGE_INIT_BALANCE,
        )
        .unwrap();

        run_failed_test_and_assert(
            EvmDataWriter::new_with_selector(Action::Swap)
                .write(H256::from(target_swap_native_account().as_ref()))
                .build(),
            U256::from(BRIDGE_INIT_BALANCE),
            expected_gas_usage,
            expected_fee,
            ExitReason::Error(ExitError::Other("account would be killed".into())),
            EvmDataWriter::new().build(),
        );
    });
}

/// This test verifies that the swap precompile call fails when a bad selector is passed.
#[test]
fn swap_fail_bad_selector() {
    new_test_ext().execute_with_ext(|_| {
        let expected_gas_usage: u64 = 50_000; // all passed gas will be consumed
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        run_failed_test_and_assert(
            EvmDataWriter::new_with_selector(111_u32)
                .write(H256::from(target_swap_native_account().as_ref()))
                .build(),
            U256::from(1),
            expected_gas_usage,
            expected_fee,
            ExitReason::Error(ExitError::Other("invalid function selector".into())),
            EvmDataWriter::new().build(),
        );
    });
}

/// This test verifies that the swap precompile call fails when the call has no
/// arguments.
#[test]
fn swap_fail_no_arguments() {
    new_test_ext().execute_with_ext(|_| {
        let expected_gas_usage: u64 = 50_000; // all passed gas will be consumed
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        run_failed_test_and_assert(
            EvmDataWriter::new_with_selector(Action::Swap).build(),
            U256::from(1),
            expected_gas_usage,
            expected_fee,
            ExitReason::Error(ExitError::Other("exactly one argument is expected".into())),
            EvmDataWriter::new().build(),
        );
    });
}

/// This test verifies that the swap precompile call fails the call has an incomplete argument.
#[test]
fn swap_fail_short_argument() {
    new_test_ext().execute_with_ext(|_| {
        let expected_gas_usage: u64 = 50_000; // all passed gas will be consumed
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        let mut input = EvmDataWriter::new_with_selector(Action::Swap).build();
        input.extend_from_slice(&hex_literal::hex!("1000")); // bad input

        run_failed_test_and_assert(
            input,
            U256::from(1),
            expected_gas_usage,
            expected_fee,
            ExitReason::Error(ExitError::Other("exactly one argument is expected".into())),
            EvmDataWriter::new().build(),
        );
    });
}

/// This test verifies that the swap precompile call fails when the call has extra data after
/// the end of the first argument.
#[test]
fn swap_fail_trailing_junk() {
    new_test_ext().execute_with_ext(|_| {
        let expected_gas_usage: u64 = 50_000; // all passed gas will be consumed
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        let mut input = EvmDataWriter::new_with_selector(Action::Swap)
            .write(H256::from(target_swap_native_account().as_ref()))
            .build();
        input.extend_from_slice(&hex_literal::hex!("1000")); // bad input

        run_failed_test_and_assert(
            input,
            U256::from(1),
            expected_gas_usage,
            expected_fee,
            ExitReason::Error(ExitError::Other("junk at the end of input".into())),
            EvmDataWriter::new().build(),
        );
    });
}

/// This test verifies that the swap precompile call fails when called without the sufficient balance.
#[test]
fn runner_fail_source_balance_no_funds() {
    new_test_ext().execute_with_ext(|_| {
        assert_noop!(
            Err::<(), DispatchError>(
                <Test as pallet_evm::Config>::Runner::call(
                    source_swap_evm_account(),
                    *PRECOMPILE_ADDRESS,
                    EvmDataWriter::new_with_selector(Action::Swap)
                        .write(H256::from(target_swap_native_account().as_ref()))
                        .build(),
                    U256::from(INIT_BALANCE + 1),
                    50_000, // a reasonable upper bound for tests
                    Some(*GAS_PRICE),
                    Some(*GAS_PRICE),
                    None,
                    Vec::new(),
                    true,
                    true,
                    None,
                    None,
                    <Test as pallet_evm::Config>::config(),
                )
                .unwrap_err()
                .error
                .into()
            ),
            pallet_evm::Error::<Test>::BalanceLow
        );
    });
}

/// This test verifies that the swap precompile call fails when the call value is overflowing the
/// underlying balance type.
/// This test actually unable to invoke the condition, as it fails prior to that error due to
/// a failing balance check. Nonetheless, this behaviour is verified in this test.
/// The test name could be misleading, but the idea here is that this test is a demonstration of how
/// we tried to test the value overflow and could not.
#[test]
fn runner_fail_value_overflow() {
    new_test_ext().execute_with_ext(|_| {
        // Invoke the function under test.
        assert_noop!(
            Err::<(), DispatchError>(
                <Test as pallet_evm::Config>::Runner::call(
                    source_swap_evm_account(),
                    *PRECOMPILE_ADDRESS,
                    EvmDataWriter::new_with_selector(Action::Swap)
                        .write(H256::from(target_swap_native_account().as_ref()))
                        .build(),
                    U256::MAX,
                    50_000, // a reasonable upper bound for tests
                    Some(*GAS_PRICE),
                    Some(*GAS_PRICE),
                    None,
                    Vec::new(),
                    true,
                    true,
                    None,
                    None,
                    <Test as pallet_evm::Config>::config(),
                )
                .unwrap_err()
                .error
                .into()
            ),
            pallet_evm::Error::<Test>::BalanceLow
        );
    });
}
