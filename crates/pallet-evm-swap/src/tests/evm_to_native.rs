use fp_ethereum::ValidatedTransaction;
use fp_evm::{ExitError, ExitReason, ExitSucceed};
use frame_support::{
    dispatch::{Pays, PostDispatchInfo},
    traits::{fungible::Unbalanced, OnFinalize},
};
use pallet_evm::GasWeightMapping;
use precompile_utils::{EvmDataWriter, LogsBuilder};
use sp_core::H256;

use crate::{
    mock::{alice_evm as source_swap_evm_account, *},
    *,
};

/// Returns target swap native account used in tests.
fn target_swap_native_account() -> AccountId {
    AccountId::from(hex_literal::hex!(
        "7700000000000000000000000000000000000000000000000000000000000077"
    ))
}

/// A helper function to run succeeded test and assert state changes.
fn run_succeeded_test_and_assert(swap_balance: Balance, expected_gas_usage: u64) {
    let source_swap_evm_account_balance_before =
        EvmBalances::total_balance(&source_swap_evm_account());
    let target_swap_native_account_balance_before =
        Balances::total_balance(&target_swap_native_account());

    // Set block number to enable events.
    System::set_block_number(1);

    // Prepare EVM call.
    let transaction = pallet_ethereum::Transaction::EIP1559(ethereum::EIP1559Transaction {
        chain_id: <Test as pallet_evm::Config>::ChainId::get(),
        nonce: pallet_evm::Pallet::<Test>::account_basic(&source_swap_evm_account())
            .0
            .nonce,
        max_priority_fee_per_gas: 0.into(),
        max_fee_per_gas: 0.into(),
        gas_limit: 50_000.into(), // a reasonable upper bound for tests
        action: ethereum::TransactionAction::Call(*PRECOMPILE_ADDRESS),
        value: U256::from(swap_balance),
        input: EvmDataWriter::new_with_selector(precompile::Action::Swap)
            .write(H256::from(target_swap_native_account().as_ref()))
            .build(),
        access_list: Default::default(),
        odd_y_parity: false,
        r: Default::default(),
        s: Default::default(),
    });
    let transaction_hash = transaction.hash();

    // Invoke the function under test.
    let post_info = pallet_ethereum::ValidatedTransaction::<Test>::apply(
        source_swap_evm_account(),
        transaction,
    )
    .unwrap();

    assert_eq!(
        post_info,
        PostDispatchInfo {
            actual_weight: Some(
                <Test as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
                    expected_gas_usage,
                    true
                )
            ),
            pays_fee: Pays::No
        },
    );

    // Assert state changes.

    // Verify that source swap evm balance has been decreased by swap value.
    assert_eq!(
        <EvmBalances>::total_balance(&source_swap_evm_account()),
        source_swap_evm_account_balance_before - swap_balance,
    );
    // Verify that bridge pot evm balance has been increased by swap value.
    assert_eq!(
        EvmBalances::total_balance(&BridgePotEvm::get()),
        BRIDGE_INIT_BALANCE + swap_balance,
    );
    // Verify that target swap native balance has been increased by swap value.
    assert_eq!(
        <Balances>::total_balance(&target_swap_native_account()),
        target_swap_native_account_balance_before + swap_balance
    );
    // Verify that bridge pot native balance has been decreased by swap value.
    assert_eq!(
        Balances::total_balance(&BridgePotNative::get()),
        BRIDGE_INIT_BALANCE - swap_balance,
    );
    // Verify that precompile balance remains the same.
    assert_eq!(EvmBalances::total_balance(&*PRECOMPILE_ADDRESS), 0);
    // Verify that we have a corresponding ethereum event about succeeded transaction.
    System::assert_has_event(RuntimeEvent::Ethereum(pallet_ethereum::Event::Executed {
        from: source_swap_evm_account(),
        to: *PRECOMPILE_ADDRESS,
        transaction_hash,
        exit_reason: ExitReason::Succeed(ExitSucceed::Returned),
        extra_data: vec![],
    }));

    // Finalize block to check transaction logs.
    Ethereum::on_finalize(1);

    // Verify that we have expected transaction log corresponding to swap execution.
    let transaction_logs = pallet_ethereum::pallet::CurrentTransactionStatuses::<Test>::get()
        .unwrap()
        .first()
        .cloned()
        .unwrap()
        .logs;
    assert_eq!(
        transaction_logs,
        vec![LogsBuilder::new(*PRECOMPILE_ADDRESS).log3(
            precompile::SELECTOR_LOG_SWAP,
            source_swap_evm_account(),
            H256::from(target_swap_native_account().as_ref()),
            EvmDataWriter::new().write(swap_balance).build(),
        )]
    );
}

/// This test verifies that the swap precompile call works in the happy path.
#[test]
fn swap_works() {
    new_test_ext().execute_with_ext(|_| {
        run_succeeded_test_and_assert(100, 21216 + 200);
    });
}

/// This test verifies that the swap precompile call works when we transfer *almost* the full
/// account balance.
/// Almost because we leave one token left on the source account.
#[test]
fn swap_works_almost_full_balance() {
    new_test_ext().execute_with_ext(|_| {
        run_succeeded_test_and_assert(INIT_BALANCE - 1, 21216 + 200);
    });
}

/// This test verifies that the swap precompile call works when we transfer the full account balance.
#[test]
fn swap_works_full_balance() {
    new_test_ext().execute_with_ext(|_| {
        run_succeeded_test_and_assert(INIT_BALANCE, 21216 + 200);
    });
}

/// A helper function to run failed test and assert state changes.
fn run_failed_test_and_assert(
    expected_gas_usage: u64,
    transaction: pallet_ethereum::Transaction,
    exit_reason: ExitReason,
) {
    let source_swap_evm_account_balance_before =
        EvmBalances::total_balance(&source_swap_evm_account());
    let target_swap_native_account_balance_before =
        Balances::total_balance(&target_swap_native_account());

    // Set block number to enable events.
    System::set_block_number(1);

    let transaction_hash = transaction.hash();

    // Invoke the function under test.
    let post_info = pallet_ethereum::ValidatedTransaction::<Test>::apply(
        source_swap_evm_account(),
        transaction,
    )
    .unwrap();

    assert_eq!(
        post_info,
        PostDispatchInfo {
            actual_weight: Some(
                <Test as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
                    expected_gas_usage,
                    true
                )
            ),
            pays_fee: Pays::No
        },
    );

    // Verify that source swap evm balance remains the same.
    assert_eq!(
        <EvmBalances>::total_balance(&source_swap_evm_account()),
        source_swap_evm_account_balance_before,
    );
    // Verify that bridge pot evm balance remains the same.
    assert_eq!(
        EvmBalances::total_balance(&BridgePotEvm::get()),
        BRIDGE_INIT_BALANCE,
    );
    // Verify that target swap native balance remains the same.
    assert_eq!(
        <Balances>::total_balance(&target_swap_native_account()),
        target_swap_native_account_balance_before
    );
    // Verify that bridge pot native balance remains the same.
    assert_eq!(
        Balances::total_balance(&BridgePotNative::get()),
        BRIDGE_INIT_BALANCE,
    );
    // Verify that precompile balance remains the same.
    assert_eq!(EvmBalances::total_balance(&*PRECOMPILE_ADDRESS), 0);
    // Verify that we have a corresponding ethereum event about failed transaction.
    System::assert_has_event(RuntimeEvent::Ethereum(pallet_ethereum::Event::Executed {
        from: source_swap_evm_account(),
        to: *PRECOMPILE_ADDRESS,
        transaction_hash,
        exit_reason,
        extra_data: vec![],
    }));

    // Finalize block to check transaction logs.
    Ethereum::on_finalize(1);

    // Verify that we don't have a transaction log corresponding to swap execution.
    assert_eq!(
        pallet_ethereum::pallet::CurrentTransactionStatuses::<Test>::get()
            .unwrap()
            .first()
            .cloned()
            .unwrap()
            .logs,
        vec![]
    );
}

/// This test verifies that the swap precompile call behaves as expected when called without
/// the sufficient balance.
#[test]
fn swap_fail_source_balance_no_funds() {
    new_test_ext().execute_with_ext(|_| {
        let swap_balance = INIT_BALANCE + 1; // more than we have
        let expected_gas_usage: u64 = 21216; // precompile gas cost is not included

        // Prepare EVM call.
        let transaction = pallet_ethereum::Transaction::EIP1559(ethereum::EIP1559Transaction {
            chain_id: <Test as pallet_evm::Config>::ChainId::get(),
            nonce: pallet_evm::Pallet::<Test>::account_basic(&source_swap_evm_account())
                .0
                .nonce,
            max_priority_fee_per_gas: 0.into(),
            max_fee_per_gas: 0.into(),
            gas_limit: 50_000.into(), // a reasonable upper bound for tests
            action: ethereum::TransactionAction::Call(*PRECOMPILE_ADDRESS),
            value: U256::from(swap_balance),
            input: EvmDataWriter::new_with_selector(precompile::Action::Swap)
                .write(H256::from(target_swap_native_account().as_ref()))
                .build(),
            access_list: Default::default(),
            odd_y_parity: false,
            r: Default::default(),
            s: Default::default(),
        });

        run_failed_test_and_assert(
            expected_gas_usage,
            transaction,
            ExitReason::Error(ExitError::OutOfFund),
        );
    });
}

/// This test verifies that the swap precompile call behaves as expected when
/// estimated swapped balance less or equal than target currency existential deposit.
#[test]
fn swap_fail_target_balance_below_ed() {
    new_test_ext().execute_with_ext(|_| {
        let swap_balance = 1;
        let expected_gas_usage: u64 = 50_000; // all gas will be consumed

        // Prepare EVM call.
        let transaction = pallet_ethereum::Transaction::EIP1559(ethereum::EIP1559Transaction {
            chain_id: <Test as pallet_evm::Config>::ChainId::get(),
            nonce: pallet_evm::Pallet::<Test>::account_basic(&source_swap_evm_account())
                .0
                .nonce,
            max_priority_fee_per_gas: 0.into(),
            max_fee_per_gas: 0.into(),
            gas_limit: 50_000.into(), // a reasonable upper bound for tests
            action: ethereum::TransactionAction::Call(*PRECOMPILE_ADDRESS),
            value: U256::from(swap_balance),
            input: EvmDataWriter::new_with_selector(precompile::Action::Swap)
                .write(H256::from(target_swap_native_account().as_ref()))
                .build(),
            access_list: Default::default(),
            odd_y_parity: false,
            r: Default::default(),
            s: Default::default(),
        });

        run_failed_test_and_assert(
            expected_gas_usage,
            transaction,
            ExitReason::Error(ExitError::Other(
                "unable to deposit into target native account: Token(BelowMinimum)".into(),
            )),
        );
    });
}

/// This test verifies that the swap precompile call behaves as expected when
/// estimated swapped balance results into target swap native account balance overflow.
#[test]
fn swap_fail_target_balance_overflow() {
    new_test_ext().execute_with_ext(|_| {
        let swap_balance = 1;
        let expected_gas_usage: u64 = 50_000; // all gas will be consumed

        Balances::write_balance(&target_swap_native_account(), Balance::MAX).unwrap();

        // Prepare EVM call.
        let transaction = pallet_ethereum::Transaction::EIP1559(ethereum::EIP1559Transaction {
            chain_id: <Test as pallet_evm::Config>::ChainId::get(),
            nonce: pallet_evm::Pallet::<Test>::account_basic(&source_swap_evm_account())
                .0
                .nonce,
            max_priority_fee_per_gas: 0.into(),
            max_fee_per_gas: 0.into(),
            gas_limit: 50_000.into(), // a reasonable upper bound for tests
            action: ethereum::TransactionAction::Call(*PRECOMPILE_ADDRESS),
            value: U256::from(swap_balance),
            input: EvmDataWriter::new_with_selector(precompile::Action::Swap)
                .write(H256::from(target_swap_native_account().as_ref()))
                .build(),
            access_list: Default::default(),
            odd_y_parity: false,
            r: Default::default(),
            s: Default::default(),
        });

        run_failed_test_and_assert(
            expected_gas_usage,
            transaction,
            ExitReason::Error(ExitError::Other(
                "unable to deposit into target native account: Arithmetic(Overflow)".into(),
            )),
        );
    });
}

/// This test verifies that the swap precompile call behaves as expected when a bad selector is
/// passed.
#[test]
fn swap_fail_bad_selector() {
    new_test_ext().execute_with_ext(|_| {
        let swap_balance = 1;
        let expected_gas_usage: u64 = 50_000; // all gas will be consumed

        // Prepare EVM call.
        let transaction = pallet_ethereum::Transaction::EIP1559(ethereum::EIP1559Transaction {
            chain_id: <Test as pallet_evm::Config>::ChainId::get(),
            nonce: pallet_evm::Pallet::<Test>::account_basic(&source_swap_evm_account())
                .0
                .nonce,
            max_priority_fee_per_gas: 0.into(),
            max_fee_per_gas: 0.into(),
            gas_limit: 50_000.into(), // a reasonable upper bound for tests
            action: ethereum::TransactionAction::Call(*PRECOMPILE_ADDRESS),
            value: U256::from(swap_balance),
            input: EvmDataWriter::new_with_selector(111_u32)
                .write(H256::from(target_swap_native_account().as_ref()))
                .build(),
            access_list: Default::default(),
            odd_y_parity: false,
            r: Default::default(),
            s: Default::default(),
        });

        run_failed_test_and_assert(
            expected_gas_usage,
            transaction,
            ExitReason::Error(ExitError::Other("invalid function selector".into())),
        );
    });
}

/// This test verifies that the swap precompile call behaves as expected when the call value
/// is overflowing the underlying balance type.
/// This test actually unable to invoke the condition, as it fails prior to that error due to
/// a failing balance check. Nonetheless, this behaviour is verified in this test.
/// The test name could be misleading, but the idea here is that this test is a demonstration of how
/// we tried to test the value overflow and could not.
#[test]
fn swap_fail_value_overflow() {
    new_test_ext().execute_with_ext(|_| {
        let expected_gas_usage: u64 = 21216; // precompile gas cost is not included

        // Prepare EVM call.
        let transaction = pallet_ethereum::Transaction::EIP1559(ethereum::EIP1559Transaction {
            chain_id: <Test as pallet_evm::Config>::ChainId::get(),
            nonce: pallet_evm::Pallet::<Test>::account_basic(&source_swap_evm_account())
                .0
                .nonce,
            max_priority_fee_per_gas: 0.into(),
            max_fee_per_gas: 0.into(),
            gas_limit: 50_000.into(), // a reasonable upper bound for tests
            action: ethereum::TransactionAction::Call(*PRECOMPILE_ADDRESS),
            value: U256::MAX, // use max value
            input: EvmDataWriter::new_with_selector(precompile::Action::Swap)
                .write(H256::from(target_swap_native_account().as_ref()))
                .build(),
            access_list: Default::default(),
            odd_y_parity: false,
            r: Default::default(),
            s: Default::default(),
        });

        run_failed_test_and_assert(
            expected_gas_usage,
            transaction,
            ExitReason::Error(ExitError::OutOfFund),
        );
    });
}

/// This test verifies that the swap precompile call behaves as expected when the call has no
/// arguments.
#[test]
fn swap_fail_no_arguments() {
    new_test_ext().execute_with_ext(|_| {
        let swap_balance = 1;
        let expected_gas_usage: u64 = 50_000; // all gas will be consumed

        // Prepare EVM call.
        let transaction = pallet_ethereum::Transaction::EIP1559(ethereum::EIP1559Transaction {
            chain_id: <Test as pallet_evm::Config>::ChainId::get(),
            nonce: pallet_evm::Pallet::<Test>::account_basic(&source_swap_evm_account())
                .0
                .nonce,
            max_priority_fee_per_gas: 0.into(),
            max_fee_per_gas: 0.into(),
            gas_limit: 50_000.into(), // a reasonable upper bound for tests
            action: ethereum::TransactionAction::Call(*PRECOMPILE_ADDRESS),
            value: U256::from(swap_balance),
            input: EvmDataWriter::new_with_selector(precompile::Action::Swap).build(),
            access_list: Default::default(),
            odd_y_parity: false,
            r: Default::default(),
            s: Default::default(),
        });

        run_failed_test_and_assert(
            expected_gas_usage,
            transaction,
            ExitReason::Error(ExitError::Other("exactly one argument is expected".into())),
        );
    });
}

/// This test verifies that the swap precompile call behaves as expected when the call has
/// an incomplete argument.
#[test]
fn swap_fail_short_argument() {
    new_test_ext().execute_with_ext(|_| {
        let swap_balance = 1;
        let expected_gas_usage: u64 = 50_000; // all gas will be consumed

        // Prepare EVM call.
        let mut input = EvmDataWriter::new_with_selector(precompile::Action::Swap).build();
        input.extend_from_slice(&hex_literal::hex!("1000")); // bad input

        let transaction = pallet_ethereum::Transaction::EIP1559(ethereum::EIP1559Transaction {
            chain_id: <Test as pallet_evm::Config>::ChainId::get(),
            nonce: pallet_evm::Pallet::<Test>::account_basic(&source_swap_evm_account())
                .0
                .nonce,
            max_priority_fee_per_gas: 0.into(),
            max_fee_per_gas: 0.into(),
            gas_limit: 50_000.into(), // a reasonable upper bound for tests
            action: ethereum::TransactionAction::Call(*PRECOMPILE_ADDRESS),
            value: U256::from(swap_balance),
            input,
            access_list: Default::default(),
            odd_y_parity: false,
            r: Default::default(),
            s: Default::default(),
        });

        run_failed_test_and_assert(
            expected_gas_usage,
            transaction,
            ExitReason::Error(ExitError::Other("exactly one argument is expected".into())),
        );
    });
}

/// This test verifies that the swap precompile call behaves as expected when the call has
/// extra data after the end of the first argument.
#[test]
fn swap_fail_trailing_junk() {
    new_test_ext().execute_with_ext(|_| {
        let swap_balance = 1;
        let expected_gas_usage: u64 = 50_000; // all gas will be consumed

        // Prepare EVM call.
        let mut input = EvmDataWriter::new_with_selector(precompile::Action::Swap)
            .write(H256::from(target_swap_native_account().as_ref()))
            .build();
        input.extend_from_slice(&hex_literal::hex!("1000")); // bad input

        let transaction = pallet_ethereum::Transaction::EIP1559(ethereum::EIP1559Transaction {
            chain_id: <Test as pallet_evm::Config>::ChainId::get(),
            nonce: pallet_evm::Pallet::<Test>::account_basic(&source_swap_evm_account())
                .0
                .nonce,
            max_priority_fee_per_gas: 0.into(),
            max_fee_per_gas: 0.into(),
            gas_limit: 50_000.into(), // a reasonable upper bound for tests
            action: ethereum::TransactionAction::Call(*PRECOMPILE_ADDRESS),
            value: U256::from(swap_balance),
            input,
            access_list: Default::default(),
            odd_y_parity: false,
            r: Default::default(),
            s: Default::default(),
        });

        run_failed_test_and_assert(
            expected_gas_usage,
            transaction,
            ExitReason::Error(ExitError::Other("junk at the end of input".into())),
        );
    });
}
