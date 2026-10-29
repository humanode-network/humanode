#![allow(clippy::arithmetic_side_effects)] // not a problem in tests

use pallet_evm::{PrecompileSet, Runner};
use precompile_utils::{testing::*, EvmDataWriter};

use crate::{mock::*, *};

fn precompiles() -> Precompiles<Test> {
    PrecompilesValue::get()
}

/// A utility that performs gas to fee computation.
/// Might not be explicitly correct, but does the job.
fn gas_to_fee(gas: u64) -> Balance {
    u128::from(gas) * u128::try_from(*GAS_PRICE).unwrap()
}

#[test]
fn metadata_name_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));

        let name_action = EvmDataWriter::new_with_selector(Action::Name).build();

        precompiles()
            .prepare_test(alice_evm, *PRECOMPILE_ADDRESS, name_action)
            .expect_cost(0)
            .expect_no_logs()
            .execute_returns(EvmDataWriter::new().write(Bytes::from(NAME)).build());
    });
}

#[test]
fn metadata_symbol_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));

        let symbol_action = EvmDataWriter::new_with_selector(Action::Symbol).build();

        precompiles()
            .prepare_test(alice_evm, *PRECOMPILE_ADDRESS, symbol_action)
            .expect_cost(0)
            .expect_no_logs()
            .execute_returns(EvmDataWriter::new().write(Bytes::from(SYMBOL)).build());
    });
}

#[test]
fn metadata_decimals_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));

        let decimals_action = EvmDataWriter::new_with_selector(Action::Decimals).build();

        precompiles()
            .prepare_test(alice_evm, *PRECOMPILE_ADDRESS, decimals_action)
            .expect_cost(0)
            .expect_no_logs()
            .execute_returns(EvmDataWriter::new().write(DECIMALS).build());
    });
}

#[test]
fn total_supply_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let bob_evm = H160::from(hex_literal::hex!(
            "7000000000000000000000000000000000000007"
        ));
        let bob_evm_balance = 200 * 10u128.pow(18);

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);
        EvmBalances::make_free_balance_be(&bob_evm, bob_evm_balance);

        let total_supply_action = EvmDataWriter::new_with_selector(Action::TotalSupply).build();

        precompiles()
            .prepare_test(bob_evm, *PRECOMPILE_ADDRESS, total_supply_action)
            .expect_cost(0)
            .expect_no_logs()
            .execute_returns(
                EvmDataWriter::new()
                    .write(U256::from(alice_evm_balance + bob_evm_balance))
                    .build(),
            );
    });
}

#[test]
fn balance_of_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let bob_evm = H160::from(hex_literal::hex!(
            "7000000000000000000000000000000000000007"
        ));

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        let balance_of_action = EvmDataWriter::new_with_selector(Action::BalanceOf)
            .write(Address::from(alice_evm))
            .build();

        precompiles()
            .prepare_test(bob_evm, *PRECOMPILE_ADDRESS, balance_of_action)
            .expect_cost(0)
            .expect_no_logs()
            .execute_returns(
                EvmDataWriter::new()
                    .write(U256::from(alice_evm_balance))
                    .build(),
            );
    });
}

#[test]
fn approve_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let bob_evm = H160::from(hex_literal::hex!(
            "7000000000000000000000000000000000000007"
        ));
        let approved_alice_bob_balance = 10 * 10u128.pow(18);
        let charlie_evm = H160::from(hex_literal::hex!(
            "9000000000000000000000000000000000000009"
        ));

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        let approve_action = EvmDataWriter::new_with_selector(Action::Approve)
            .write(Address::from(bob_evm))
            .write(U256::from(approved_alice_bob_balance))
            .build();

        precompiles()
            .prepare_test(alice_evm, *PRECOMPILE_ADDRESS, approve_action)
            .expect_cost(GAS_COST)
            .expect_log(
                LogsBuilder::new(*PRECOMPILE_ADDRESS).log3(
                    SELECTOR_LOG_APPROVAL,
                    alice_evm,
                    bob_evm,
                    EvmDataWriter::new()
                        .write(approved_alice_bob_balance)
                        .build(),
                ),
            )
            .execute_returns(EvmDataWriter::new().write(true).build());

        let allowance_action = EvmDataWriter::new_with_selector(Action::Allowance)
            .write(Address::from(alice_evm))
            .write(Address::from(bob_evm))
            .build();

        precompiles()
            .prepare_test(charlie_evm, *PRECOMPILE_ADDRESS, allowance_action)
            .expect_cost(0)
            .expect_no_logs()
            .execute_returns(
                EvmDataWriter::new()
                    .write(U256::from(approved_alice_bob_balance))
                    .build(),
            );
    });
}

#[test]
fn transfer_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let bob_evm = H160::from(hex_literal::hex!(
            "7000000000000000000000000000000000000007"
        ));
        let alice_bob_transfer_balance = 10 * 10u128.pow(18);

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        let transfer_action = EvmDataWriter::new_with_selector(Action::Transfer)
            .write(Address::from(bob_evm))
            .write(U256::from(alice_bob_transfer_balance))
            .build();

        precompiles()
            .prepare_test(alice_evm, *PRECOMPILE_ADDRESS, transfer_action)
            .expect_cost(GAS_COST)
            .expect_log(
                LogsBuilder::new(*PRECOMPILE_ADDRESS).log3(
                    SELECTOR_LOG_TRANSFER,
                    alice_evm,
                    bob_evm,
                    EvmDataWriter::new()
                        .write(alice_bob_transfer_balance)
                        .build(),
                ),
            )
            .execute_returns(EvmDataWriter::new().write(true).build());

        assert_eq!(
            EvmBalances::total_balance(&alice_evm),
            alice_evm_balance - alice_bob_transfer_balance
        );
        assert_eq!(
            EvmBalances::total_balance(&bob_evm),
            alice_bob_transfer_balance
        );
    });
}

#[test]
fn transfer_from_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let bob_evm = H160::from(hex_literal::hex!(
            "7000000000000000000000000000000000000007"
        ));
        let approved_alice_bob_balance = 10 * 10u128.pow(18);
        let charlie_evm = H160::from(hex_literal::hex!(
            "9000000000000000000000000000000000000009"
        ));
        let bob_charlie_transfer_from_alice_balance = 5 * 10u128.pow(18);

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        let approve_action = EvmDataWriter::new_with_selector(Action::Approve)
            .write(Address::from(bob_evm))
            .write(U256::from(approved_alice_bob_balance))
            .build();

        precompiles()
            .prepare_test(alice_evm, *PRECOMPILE_ADDRESS, approve_action)
            .expect_cost(GAS_COST)
            .expect_log(
                LogsBuilder::new(*PRECOMPILE_ADDRESS).log3(
                    SELECTOR_LOG_APPROVAL,
                    alice_evm,
                    bob_evm,
                    EvmDataWriter::new()
                        .write(approved_alice_bob_balance)
                        .build(),
                ),
            )
            .execute_returns(EvmDataWriter::new().write(true).build());

        let transfer_from_action = EvmDataWriter::new_with_selector(Action::TransferFrom)
            .write(Address::from(alice_evm))
            .write(Address::from(charlie_evm))
            .write(U256::from(bob_charlie_transfer_from_alice_balance))
            .build();

        precompiles()
            .prepare_test(bob_evm, *PRECOMPILE_ADDRESS, transfer_from_action)
            .expect_cost(GAS_COST)
            .expect_log(
                LogsBuilder::new(*PRECOMPILE_ADDRESS).log3(
                    SELECTOR_LOG_TRANSFER,
                    alice_evm,
                    charlie_evm,
                    EvmDataWriter::new()
                        .write(bob_charlie_transfer_from_alice_balance)
                        .build(),
                ),
            )
            .execute_returns(EvmDataWriter::new().write(true).build());

        assert_eq!(
            EvmBalances::total_balance(&alice_evm),
            alice_evm_balance - bob_charlie_transfer_from_alice_balance
        );
        assert_eq!(EvmBalances::total_balance(&bob_evm), 0);
        assert_eq!(
            EvmBalances::total_balance(&charlie_evm),
            bob_charlie_transfer_from_alice_balance
        );
    });
}

#[test]
fn transfer_from_fails_spend_more_than_allowed() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let bob_evm = H160::from(hex_literal::hex!(
            "7000000000000000000000000000000000000007"
        ));
        let approved_alice_bob_balance = 2 * 10u128.pow(18);
        let charlie_evm = H160::from(hex_literal::hex!(
            "9000000000000000000000000000000000000009"
        ));
        let bob_charlie_transfer_from_alice_balance = 5 * 10u128.pow(18);

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        let approve_action = EvmDataWriter::new_with_selector(Action::Approve)
            .write(Address::from(bob_evm))
            .write(U256::from(approved_alice_bob_balance))
            .build();

        precompiles()
            .prepare_test(alice_evm, *PRECOMPILE_ADDRESS, approve_action)
            .expect_cost(GAS_COST)
            .expect_log(
                LogsBuilder::new(*PRECOMPILE_ADDRESS).log3(
                    SELECTOR_LOG_APPROVAL,
                    alice_evm,
                    bob_evm,
                    EvmDataWriter::new()
                        .write(approved_alice_bob_balance)
                        .build(),
                ),
            )
            .execute_returns(EvmDataWriter::new().write(true).build());

        let transfer_from_action = EvmDataWriter::new_with_selector(Action::TransferFrom)
            .write(Address::from(alice_evm))
            .write(Address::from(charlie_evm))
            .write(U256::from(bob_charlie_transfer_from_alice_balance))
            .build();

        precompiles()
            .prepare_test(bob_evm, *PRECOMPILE_ADDRESS, transfer_from_action)
            .expect_cost(GAS_COST)
            .expect_no_logs()
            .execute_error(ExitError::Other("spend more than allowed".into()));

        assert_eq!(EvmBalances::total_balance(&alice_evm), alice_evm_balance);
        assert_eq!(EvmBalances::total_balance(&bob_evm), 0);
        assert_eq!(EvmBalances::total_balance(&charlie_evm), 0);
    });
}

#[test]
fn deposit_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let alice_deposit_balance = 10 * 10u128.pow(18);

        let expected_gas_usage: u64 = 21264;
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        let deposit_action = EvmDataWriter::new_with_selector(Action::Deposit).build();

        // Invoke deposit.
        // We need to call using EVM runner so we can check the EVM correctly sends the amount
        // to the precompile.
        let config = <Test as pallet_evm::Config>::config();
        let execinfo = <Test as pallet_evm::Config>::Runner::call(
            alice_evm,
            *PRECOMPILE_ADDRESS,
            deposit_action,
            alice_deposit_balance.into(),
            50_000, // a reasonable upper bound for tests
            Some(*GAS_PRICE),
            Some(*GAS_PRICE),
            None,
            Vec::new(),
            true,
            true,
            None,
            None,
            config,
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
            vec![LogsBuilder::new(*PRECOMPILE_ADDRESS).log2(
                SELECTOR_LOG_DEPOSIT,
                alice_evm,
                EvmDataWriter::new().write(alice_deposit_balance).build(),
            )]
        );

        assert_eq!(
            EvmBalances::total_balance(&alice_evm),
            alice_evm_balance - expected_fee
        );
    });
}

#[test]
fn deposit_fails_zero_value() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let alice_deposit_balance = 0;

        let expected_gas_usage: u64 = 21264;
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        let deposit_action = EvmDataWriter::new_with_selector(Action::Deposit).build();

        // Invoke deposit.
        // We need to call using EVM runner so we can check the EVM correctly sends the amount
        // to the precompile.
        let config = <Test as pallet_evm::Config>::config();
        let execinfo = <Test as pallet_evm::Config>::Runner::call(
            alice_evm,
            *PRECOMPILE_ADDRESS,
            deposit_action,
            alice_deposit_balance.into(),
            50_000, // a reasonable upper bound for tests
            Some(*GAS_PRICE),
            Some(*GAS_PRICE),
            None,
            Vec::new(),
            true,
            true,
            None,
            None,
            config,
        )
        .unwrap();
        assert_eq!(
            execinfo.exit_reason,
            fp_evm::ExitReason::Revert(ExitRevert::Reverted)
        );
        assert_eq!(execinfo.used_gas.standard, expected_gas_usage.into());
        assert_eq!(
            execinfo.value,
            "deposited amount must be non-zero".as_bytes()
        );
        assert_eq!(execinfo.logs, Vec::new());

        assert_eq!(
            EvmBalances::total_balance(&alice_evm),
            alice_evm_balance - expected_fee
        );
    });
}

#[test]
fn withdraw_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let alice_withdraw_balance = 10 * 10u128.pow(18);

        let expected_gas_usage: u64 = 21464;
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        let withdraw_action = EvmDataWriter::new_with_selector(Action::Withdraw)
            .write(U256::from(alice_withdraw_balance))
            .build();

        // Invoke withdraw.
        let config = <Test as pallet_evm::Config>::config();
        let execinfo = <Test as pallet_evm::Config>::Runner::call(
            alice_evm,
            *PRECOMPILE_ADDRESS,
            withdraw_action,
            0.into(),
            50_000, // a reasonable upper bound for tests
            Some(*GAS_PRICE),
            Some(*GAS_PRICE),
            None,
            Vec::new(),
            true,
            true,
            None,
            None,
            config,
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
            vec![LogsBuilder::new(*PRECOMPILE_ADDRESS).log2(
                SELECTOR_LOG_WITHDRAWAL,
                alice_evm,
                EvmDataWriter::new().write(alice_withdraw_balance).build(),
            )]
        );

        assert_eq!(
            EvmBalances::total_balance(&alice_evm),
            alice_evm_balance - expected_fee
        );
    });
}

#[test]
fn withdraw_fails_more_than_allowed() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 2 * 10u128.pow(18);
        let alice_withdraw_balance = 5 * 10u128.pow(18);

        let expected_gas_usage: u64 = 21464;
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        let withdraw_action = EvmDataWriter::new_with_selector(Action::Withdraw)
            .write(U256::from(alice_withdraw_balance))
            .build();

        // Invoke withdraw.
        let config = <Test as pallet_evm::Config>::config();
        let execinfo = <Test as pallet_evm::Config>::Runner::call(
            alice_evm,
            *PRECOMPILE_ADDRESS,
            withdraw_action,
            0.into(),
            50_000, // a reasonable upper bound for tests
            Some(*GAS_PRICE),
            Some(*GAS_PRICE),
            None,
            Vec::new(),
            true,
            true,
            None,
            None,
            config,
        )
        .unwrap();
        assert_eq!(
            execinfo.exit_reason,
            fp_evm::ExitReason::Revert(ExitRevert::Reverted)
        );
        assert_eq!(execinfo.used_gas.standard, expected_gas_usage.into());
        assert_eq!(
            execinfo.value,
            "trying to withdraw more than owned".as_bytes()
        );
        assert_eq!(execinfo.logs, Vec::new());

        assert_eq!(
            EvmBalances::total_balance(&alice_evm),
            alice_evm_balance - expected_fee
        );
    });
}

/// The state-changing actions, each with a well-formed input.
fn state_changing_actions(alice_evm: H160, bob_evm: H160) -> Vec<(Action, Vec<u8>)> {
    let amount = U256::from(10 * 10u128.pow(18));
    vec![
        (
            Action::Approve,
            EvmDataWriter::new_with_selector(Action::Approve)
                .write(Address::from(bob_evm))
                .write(amount)
                .build(),
        ),
        (
            Action::Transfer,
            EvmDataWriter::new_with_selector(Action::Transfer)
                .write(Address::from(bob_evm))
                .write(amount)
                .build(),
        ),
        (
            Action::TransferFrom,
            EvmDataWriter::new_with_selector(Action::TransferFrom)
                .write(Address::from(alice_evm))
                .write(Address::from(bob_evm))
                .write(amount)
                .build(),
        ),
        (
            Action::Deposit,
            EvmDataWriter::new_with_selector(Action::Deposit).build(),
        ),
        (
            Action::Withdraw,
            EvmDataWriter::new_with_selector(Action::Withdraw)
                .write(amount)
                .build(),
        ),
    ]
}

/// The view actions, each with a well-formed input.
fn view_actions(alice_evm: H160, bob_evm: H160) -> Vec<(Action, Vec<u8>)> {
    vec![
        (
            Action::Name,
            EvmDataWriter::new_with_selector(Action::Name).build(),
        ),
        (
            Action::Symbol,
            EvmDataWriter::new_with_selector(Action::Symbol).build(),
        ),
        (
            Action::Decimals,
            EvmDataWriter::new_with_selector(Action::Decimals).build(),
        ),
        (
            Action::TotalSupply,
            EvmDataWriter::new_with_selector(Action::TotalSupply).build(),
        ),
        (
            Action::BalanceOf,
            EvmDataWriter::new_with_selector(Action::BalanceOf)
                .write(Address::from(alice_evm))
                .build(),
        ),
        (
            Action::Allowance,
            EvmDataWriter::new_with_selector(Action::Allowance)
                .write(Address::from(alice_evm))
                .write(Address::from(bob_evm))
                .build(),
        ),
    ]
}

/// Build a mock handle for a call from `caller` with the given `input`.
fn mock_handle(caller: H160, input: Vec<u8>) -> MockHandle {
    let mut handle = MockHandle::new(
        *PRECOMPILE_ADDRESS,
        fp_evm::Context {
            address: *PRECOMPILE_ADDRESS,
            caller,
            apparent_value: U256::zero(),
        },
    );
    handle.input = input;
    handle
}

/// Run the precompile with the given handle and assert it reverts with `reason` without
/// consuming gas or emitting logs.
fn assert_reverts(handle: &mut MockHandle, reason: &[u8], action: Action) {
    assert_eq!(
        precompiles().execute(handle),
        Some(Err(PrecompileFailure::Revert {
            exit_status: ExitRevert::Reverted,
            output: reason.to_vec(),
        })),
        "unexpected result for {action:?}"
    );
    assert_eq!(handle.gas_used, 0, "gas consumed for {action:?}");
    assert!(handle.logs.is_empty(), "logs emitted for {action:?}");
}

/// This test verifies that the view functions are callable in a static context.
#[test]
fn view_functions_work_in_static_context() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let bob_evm = H160::from(hex_literal::hex!(
            "7000000000000000000000000000000000000007"
        ));

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        let balance_of_action = EvmDataWriter::new_with_selector(Action::BalanceOf)
            .write(Address::from(alice_evm))
            .build();

        let mut handle = mock_handle(bob_evm, balance_of_action);
        handle.is_static = true;

        assert_eq!(
            precompiles().execute(&mut handle),
            Some(Ok(succeed(
                EvmDataWriter::new()
                    .write(U256::from(alice_evm_balance))
                    .build()
            )))
        );
    });
}

/// This test verifies that the state-changing functions revert in a static context (i.e. when
/// reached via `STATICCALL`) without touching the state.
#[test]
fn state_changing_functions_fail_in_static_context() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let bob_evm = H160::from(hex_literal::hex!(
            "7000000000000000000000000000000000000007"
        ));

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);
        Erc20::approve(alice_evm, bob_evm, U256::MAX);

        for (action, input) in state_changing_actions(alice_evm, bob_evm) {
            let mut handle = mock_handle(alice_evm, input);
            handle.is_static = true;
            assert_reverts(&mut handle, b"cannot be called with STATICCALL", action);
        }

        assert_eq!(EvmBalances::total_balance(&alice_evm), alice_evm_balance);
        assert_eq!(EvmBalances::total_balance(&bob_evm), 0);
        assert_eq!(Erc20::allowance(&alice_evm, &bob_evm), U256::MAX);
    });
}

/// This test verifies that the functions that are not payable revert when value is sent along
/// with the call, so the value does not end up stuck at the precompile address.
#[test]
fn non_payable_functions_fail_with_value() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let bob_evm = H160::from(hex_literal::hex!(
            "7000000000000000000000000000000000000007"
        ));

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        let balance_of_action = EvmDataWriter::new_with_selector(Action::BalanceOf)
            .write(Address::from(alice_evm))
            .build();

        let non_payable_actions = state_changing_actions(alice_evm, bob_evm)
            .into_iter()
            .filter(|(action, _)| *action != Action::Deposit)
            .chain([(Action::BalanceOf, balance_of_action)]);

        for (action, input) in non_payable_actions {
            let mut handle = mock_handle(alice_evm, input);
            handle.context.apparent_value = U256::from(1);
            assert_reverts(&mut handle, b"function is not payable", action);
        }
    });
}

/// This test verifies that all the functions revert when reached via `DELEGATECALL` or
/// `CALLCODE` (i.e. when the context address differs from the precompile code address) without
/// touching the state.
#[test]
fn all_functions_fail_in_delegate_call() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let bob_evm = H160::from(hex_literal::hex!(
            "7000000000000000000000000000000000000007"
        ));
        let delegating_contract_evm = H160::from(hex_literal::hex!(
            "f0f0f00000000000000000000000000000f0f0f0"
        ));

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);
        Erc20::approve(alice_evm, bob_evm, U256::MAX);

        let actions = view_actions(alice_evm, bob_evm)
            .into_iter()
            .chain(state_changing_actions(alice_evm, bob_evm));

        for (action, input) in actions {
            let mut handle = mock_handle(alice_evm, input);
            // Under `DELEGATECALL` the context address is the delegating contract.
            handle.context.address = delegating_contract_evm;
            assert_reverts(
                &mut handle,
                b"cannot be called with DELEGATECALL or CALLCODE",
                action,
            );
        }

        assert_eq!(EvmBalances::total_balance(&alice_evm), alice_evm_balance);
        assert_eq!(EvmBalances::total_balance(&bob_evm), 0);
        assert_eq!(Erc20::allowance(&alice_evm, &bob_evm), U256::MAX);
    });
}
