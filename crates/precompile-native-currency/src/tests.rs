#![allow(clippy::integer_arithmetic)] // not a problem in tests

use precompile_utils::{testing::*, EvmDataWriter};

use crate::{mock::*, *};

fn precompiles() -> Precompiles<Test> {
    PrecompilesValue::get()
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
                LogsBuilder::new(*PRECOMPILE_ADDRESS).log4(
                    SELECTOR_LOG_TRANSFER,
                    bob_evm,
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
