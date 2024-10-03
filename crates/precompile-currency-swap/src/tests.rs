#![allow(clippy::arithmetic_side_effects)] // not a problem in tests

use mockall::predicate;
use pallet_evm::Runner;
use precompile_utils::EvmDataWriter;

use crate::{mock::*, *};

/// A utility that performs gas to fee computation.
/// Might not be explicitly correct, but does the job.
fn gas_to_fee(gas: u64) -> Balance {
    u128::from(gas) * u128::try_from(*GAS_PRICE).unwrap()
}

/// This test verifies that the swap precompile call works in the happy path.
#[test]
fn swap_works() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice = AccountId::from(hex_literal::hex!(
            "1000000000000000000000000000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let swap_balance = 10 * 10u128.pow(18);

        let expected_gas_usage: u64 = 21216 + 200;
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        // Check test preconditions.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);

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
                <EvmBalances as Currency<EvmAccountId>>::NegativeImbalance::new(swap_balance),
            ))
            .return_once(move |_| {
                Ok(<Balances as Currency<AccountId>>::NegativeImbalance::new(
                    swap_balance,
                ))
            });

        // Prepare EVM call.
        let input = EvmDataWriter::new_with_selector(Action::Swap)
            .write(H256::from(alice.as_ref()))
            .build();

        // Invoke the function under test.
        let config = <Test as pallet_evm::Config>::config();
        let execinfo = <Test as pallet_evm::Config>::Runner::call(
            alice_evm,
            *PRECOMPILE_ADDRESS,
            input,
            swap_balance.into(),
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
            vec![LogsBuilder::new(*PRECOMPILE_ADDRESS).log3(
                SELECTOR_LOG_SWAP,
                alice_evm,
                H256::from(alice.as_ref()),
                EvmDataWriter::new().write(swap_balance).build(),
            )]
        );

        // Assert state changes.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance - swap_balance - expected_fee
        );
        assert_eq!(
            <Balances as Currency<_>>::total_balance(&alice),
            swap_balance
        );
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&PRECOMPILE_ADDRESS),
            0
        );

        // Assert mock invocations.
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
    });
}

/// This test verifies that the swap precompile call works when we transfer *almost* the full
/// account balance.
/// Almost because we leave one token left on the source account.
#[test]
fn swap_works_almost_full_balance() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice = AccountId::from(hex_literal::hex!(
            "1000000000000000000000000000000000000000000000000000000000000001"
        ));

        let expected_gas_usage: u64 = 21216 + 200;
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        let alice_evm_balance = 100 * 10u128.pow(18);
        let swap_balance = 100 * 10u128.pow(18) - expected_fee - 1;

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        // Check test preconditions.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);

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
                <EvmBalances as Currency<EvmAccountId>>::NegativeImbalance::new(swap_balance),
            ))
            .return_once(move |_| {
                Ok(<Balances as Currency<AccountId>>::NegativeImbalance::new(
                    swap_balance,
                ))
            });

        // Prepare EVM call.
        let input = EvmDataWriter::new_with_selector(Action::Swap)
            .write(H256::from(alice.as_ref()))
            .build();

        // Invoke the function under test.
        let config = <Test as pallet_evm::Config>::config();
        let execinfo = <Test as pallet_evm::Config>::Runner::call(
            alice_evm,
            *PRECOMPILE_ADDRESS,
            input,
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
            vec![LogsBuilder::new(*PRECOMPILE_ADDRESS).log3(
                SELECTOR_LOG_SWAP,
                alice_evm,
                H256::from(alice.as_ref()),
                EvmDataWriter::new().write(swap_balance).build(),
            )]
        );

        // Assert state changes.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance - swap_balance - expected_fee
        );
        assert_eq!(<EvmBalances as Currency<_>>::total_balance(&alice_evm), 1);
        assert_eq!(
            <Balances as Currency<_>>::total_balance(&alice),
            swap_balance
        );
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&PRECOMPILE_ADDRESS),
            0
        );

        // Assert mock invocations.
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
    });
}

/// This test verifies that the swap precompile call behaves as expected when called without
/// the sufficient balance.
/// The fee is not consumed, and neither is the value.
#[test]
fn swap_fail_no_funds() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice = AccountId::from(hex_literal::hex!(
            "1000000000000000000000000000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let swap_balance = 1000 * 10u128.pow(18); // more than we have

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        // Check test preconditions.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Set mock expectations.
        let estimate_swapped_balance_ctx = MockCurrencySwap::estimate_swapped_balance_context();
        estimate_swapped_balance_ctx.expect().never();
        let swap_ctx = MockCurrencySwap::swap_context();
        swap_ctx.expect().never();

        // Prepare EVM call.
        let input = EvmDataWriter::new_with_selector(Action::Swap)
            .write(H256::from(alice.as_ref()))
            .build();

        // Invoke the function under test.
        let config = <Test as pallet_evm::Config>::config();

        let storage_root = frame_support::storage_root(sp_runtime::StateVersion::V1);
        let execerr = <Test as pallet_evm::Config>::Runner::call(
            alice_evm,
            *PRECOMPILE_ADDRESS,
            input,
            swap_balance.into(),
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
        .unwrap_err();
        assert!(matches!(execerr.error, pallet_evm::Error::BalanceLow));
        assert_eq!(
            storage_root,
            frame_support::storage_root(sp_runtime::StateVersion::V1),
            "storage changed"
        );

        // Assert state changes.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&PRECOMPILE_ADDRESS),
            0
        );

        // Assert mock invocations.
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
    });
}

/// This test verifies that the swap precompile call behaves as expected when
/// estimated swapped balance less or equal than target currency existential deposit.
/// All fee (up to specified max fee limit!) will be consumed, but not the value.
#[test]
fn swap_fail_below_ed() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice = AccountId::from(hex_literal::hex!(
            "1000000000000000000000000000000000000000000000000000000000000001"
        ));

        let expected_gas_usage: u64 = 50_123; // all fee will be consumed
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        let alice_evm_balance = 100 * 10u128.pow(18);
        let swap_balance = 10 * 10u128.pow(18);

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        // Check test preconditions.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Set mock expectations.
        let estimate_swapped_balance_ctx = MockCurrencySwap::estimate_swapped_balance_context();
        estimate_swapped_balance_ctx
            .expect()
            .once()
            .with(predicate::eq(swap_balance))
            .return_const(1_u128);
        let swap_ctx = MockCurrencySwap::swap_context();
        swap_ctx.expect().never();

        // Prepare EVM call.
        let input = EvmDataWriter::new_with_selector(Action::Swap)
            .write(H256::from(alice.as_ref()))
            .build();

        // Invoke the function under test.
        let config = <Test as pallet_evm::Config>::config();
        let execinfo = <Test as pallet_evm::Config>::Runner::call(
            alice_evm,
            *PRECOMPILE_ADDRESS,
            input,
            swap_balance.into(),
            50_123, // a reasonable upper bound for tests
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
            fp_evm::ExitReason::Error(fp_evm::ExitError::OutOfFund)
        );
        assert_eq!(execinfo.used_gas.standard, expected_gas_usage.into());
        assert_eq!(execinfo.value, Vec::<u8>::new());
        assert_eq!(execinfo.logs, Vec::new());

        // Assert state changes.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance - expected_fee
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&PRECOMPILE_ADDRESS),
            0
        );

        // Assert mock invocations.
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
    });
}

/// This test verifies that the swap precompile call behaves as expected when the currency swap
/// implementation fails.
/// The fee is consumed (and not all of it - just what was actually used), but the value is not.
/// The error message is checked to be correct.
#[test]
fn swap_fail_trait_error() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice = AccountId::from(hex_literal::hex!(
            "1000000000000000000000000000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let swap_balance = 10 * 10u128.pow(18);

        let expected_gas_usage: u64 = 21216 + 200;
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        // Check test preconditions.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);

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
                <EvmBalances as Currency<EvmAccountId>>::NegativeImbalance::new(swap_balance),
            ))
            .return_once(move |incoming_imbalance| {
                Err(primitives_currency_swap::Error {
                    cause: sp_runtime::DispatchError::Other("test"),
                    incoming_imbalance,
                })
            });

        // Prepare EVM call.
        let input = EvmDataWriter::new_with_selector(Action::Swap)
            .write(H256::from(alice.as_ref()))
            .build();

        // Invoke the function under test.
        let config = <Test as pallet_evm::Config>::config();

        let execinfo = <Test as pallet_evm::Config>::Runner::call(
            alice_evm,
            *PRECOMPILE_ADDRESS,
            input,
            swap_balance.into(),
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
        assert_eq!(execinfo.value, "unable to swap the currency".as_bytes());
        assert_eq!(execinfo.logs, Vec::new());

        // Assert state changes.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance - expected_fee
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&PRECOMPILE_ADDRESS),
            0
        );

        // Assert mock invocations.
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
    });
}

/// This test verifies that the swap precompile call works when we transfer the full account balance.
#[test]
fn swap_works_full_balance() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice = AccountId::from(hex_literal::hex!(
            "1000000000000000000000000000000000000000000000000000000000000001"
        ));

        let expected_gas_usage: u64 = 21216 + 200;
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        let alice_evm_balance = 100 * 10u128.pow(18);
        let swap_balance = 100 * 10u128.pow(18) - expected_fee;

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        // Check test preconditions.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);

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
                <EvmBalances as Currency<EvmAccountId>>::NegativeImbalance::new(swap_balance),
            ))
            .return_once(move |_| {
                Ok(<Balances as Currency<AccountId>>::NegativeImbalance::new(
                    swap_balance,
                ))
            });

        // Prepare EVM call.
        let input = EvmDataWriter::new_with_selector(Action::Swap)
            .write(H256::from(alice.as_ref()))
            .build();

        // Invoke the function under test.
        let config = <Test as pallet_evm::Config>::config();

        let execinfo = <Test as pallet_evm::Config>::Runner::call(
            alice_evm,
            *PRECOMPILE_ADDRESS,
            input,
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
            vec![LogsBuilder::new(*PRECOMPILE_ADDRESS).log3(
                SELECTOR_LOG_SWAP,
                alice_evm,
                H256::from(alice.as_ref()),
                EvmDataWriter::new().write(swap_balance).build(),
            )]
        );

        // Assert state changes.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance - swap_balance - expected_fee
        );
        assert_eq!(<EvmBalances as Currency<_>>::total_balance(&alice_evm), 0);
        assert_eq!(
            <Balances as Currency<_>>::total_balance(&alice),
            swap_balance
        );
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&PRECOMPILE_ADDRESS),
            0
        );

        // Assert mock invocations.
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
    });
}

/// This test verifies that the swap precompile call behaves as expected when a bad selector is
/// passed.
/// All fee (up to specified max fee limit!) will be consumed, but not the value.
#[test]
fn swap_fail_bad_selector() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice = AccountId::from(hex_literal::hex!(
            "1000000000000000000000000000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let swap_balance = 10 * 10u128.pow(18);

        let expected_gas_usage: u64 = 50_123; // all fee will be consumed
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        // Check test preconditions.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Set mock expectations.
        let estimate_swapped_balance_ctx = MockCurrencySwap::estimate_swapped_balance_context();
        estimate_swapped_balance_ctx.expect().never();
        let swap_ctx = MockCurrencySwap::swap_context();
        swap_ctx.expect().never();

        // Prepare EVM call.
        let input = EvmDataWriter::new_with_selector(123u32)
            .write(H256::from(alice.as_ref()))
            .build();

        // Invoke the function under test.
        let config = <Test as pallet_evm::Config>::config();
        let execinfo = <Test as pallet_evm::Config>::Runner::call(
            alice_evm,
            *PRECOMPILE_ADDRESS,
            input,
            swap_balance.into(),
            50_123, // a reasonable upper bound for tests
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
            fp_evm::ExitReason::Error(fp_evm::ExitError::Other("invalid function selector".into()))
        );
        assert_eq!(execinfo.used_gas.standard, expected_gas_usage.into());
        assert_eq!(execinfo.value, Vec::<u8>::new());
        assert_eq!(execinfo.logs, Vec::new());

        // Assert state changes.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance - expected_fee
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&PRECOMPILE_ADDRESS),
            0
        );

        // Assert mock invocations.
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
    });
}

/// This test verifies that the swap precompile call behaves as expected when the call value
/// is overflowing the underlying balance type.
/// This test actually unable to invoke the condition, as it fails prior to that error due to
/// a failing balance check. Nonetheless, this behaviour is verified in this test.
/// The test name could be misleading, but the idea here is that this test is a demonstration of how
/// we tried to test the value overflow and could not.
/// Fee will be consumed, but not the value.
#[test]
fn swap_fail_value_overflow() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice = AccountId::from(hex_literal::hex!(
            "1000000000000000000000000000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = u128::MAX;
        let swap_balance_u256: U256 = U256::from(u128::MAX) + U256::from(1);

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        // Check test preconditions.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Set mock expectations.
        let estimate_swapped_balance_ctx = MockCurrencySwap::estimate_swapped_balance_context();
        estimate_swapped_balance_ctx.expect().never();
        let swap_ctx = MockCurrencySwap::swap_context();
        swap_ctx.expect().never();

        // Prepare EVM call.
        let input = EvmDataWriter::new_with_selector(123u32)
            .write(H256::from(alice.as_ref()))
            .build();

        // Invoke the function under test.
        let config = <Test as pallet_evm::Config>::config();
        let storage_root = frame_support::storage_root(sp_runtime::StateVersion::V1);
        let execerr = <Test as pallet_evm::Config>::Runner::call(
            alice_evm,
            *PRECOMPILE_ADDRESS,
            input,
            swap_balance_u256,
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
        .unwrap_err();
        assert!(matches!(execerr.error, pallet_evm::Error::BalanceLow));
        assert_eq!(
            storage_root,
            frame_support::storage_root(sp_runtime::StateVersion::V1),
            "storage changed"
        );

        // Assert state changes.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&PRECOMPILE_ADDRESS),
            0
        );

        // Assert mock invocations.
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
    });
}

/// This test verifies that the swap precompile call behaves as expected when the call has no
/// arguments.
/// All fee (up to specified max fee limit!) will be consumed, but not the value.
#[test]
fn swap_fail_no_arguments() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice = AccountId::from(hex_literal::hex!(
            "1000000000000000000000000000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let swap_balance = 10 * 10u128.pow(18);

        let expected_gas_usage: u64 = 50_123; // all fee will be consumed
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        // Check test preconditions.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Set mock expectations.
        let estimate_swapped_balance_ctx = MockCurrencySwap::estimate_swapped_balance_context();
        estimate_swapped_balance_ctx.expect().never();
        let swap_ctx = MockCurrencySwap::swap_context();
        swap_ctx.expect().never();

        // Prepare EVM call.
        let input = EvmDataWriter::new_with_selector(Action::Swap).build();

        // Invoke the function under test.
        let config = <Test as pallet_evm::Config>::config();
        let execinfo = <Test as pallet_evm::Config>::Runner::call(
            alice_evm,
            *PRECOMPILE_ADDRESS,
            input,
            swap_balance.into(),
            50_123, // a reasonable upper bound for tests
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
            fp_evm::ExitReason::Error(fp_evm::ExitError::Other(
                "exactly one argument is expected".into()
            ))
        );
        assert_eq!(execinfo.used_gas.standard, expected_gas_usage.into());
        assert_eq!(execinfo.value, Vec::<u8>::new());
        assert_eq!(execinfo.logs, Vec::new());

        // Assert state changes.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance - expected_fee
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&PRECOMPILE_ADDRESS),
            0
        );

        // Assert mock invocations.
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
    });
}

/// This test verifies that the swap precompile call behaves as expected when the call has
/// an incomplete argument.
/// All fee (up to specified max fee limit!) will be consumed, but not the value.
#[test]
fn swap_fail_short_argument() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice = AccountId::from(hex_literal::hex!(
            "1000000000000000000000000000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let swap_balance = 10 * 10u128.pow(18);

        let expected_gas_usage: u64 = 50_123; // all fee will be consumed
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        // Check test preconditions.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Set mock expectations.
        let estimate_swapped_balance_ctx = MockCurrencySwap::estimate_swapped_balance_context();
        estimate_swapped_balance_ctx.expect().never();
        let swap_ctx = MockCurrencySwap::swap_context();
        swap_ctx.expect().never();

        // Prepare EVM call.
        let mut input = EvmDataWriter::new_with_selector(Action::Swap).build();
        input.extend_from_slice(&hex_literal::hex!("1000")); // bad input

        // Invoke the function under test.
        let config = <Test as pallet_evm::Config>::config();
        let execinfo = <Test as pallet_evm::Config>::Runner::call(
            alice_evm,
            *PRECOMPILE_ADDRESS,
            input,
            swap_balance.into(),
            50_123, // a reasonable upper bound for tests
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
            fp_evm::ExitReason::Error(fp_evm::ExitError::Other(
                "exactly one argument is expected".into()
            ))
        );
        assert_eq!(execinfo.used_gas.standard, expected_gas_usage.into());
        assert_eq!(execinfo.value, Vec::<u8>::new());
        assert_eq!(execinfo.logs, Vec::new());

        // Assert state changes.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance - expected_fee
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&PRECOMPILE_ADDRESS),
            0
        );

        // Assert mock invocations.
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
    });
}

/// This test verifies that the swap precompile call behaves as expected when the call has
/// extra data after the end of the first argument.
/// All fee (up to specified max fee limit!) will be consumed, but not the value.
#[test]
fn swap_fail_trailing_junk() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice = AccountId::from(hex_literal::hex!(
            "1000000000000000000000000000000000000000000000000000000000000001"
        ));
        let alice_evm_balance = 100 * 10u128.pow(18);
        let swap_balance = 10 * 10u128.pow(18);

        let expected_gas_usage: u64 = 50_123; // all fee will be consumed
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_evm_balance);

        // Check test preconditions.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Set mock expectations.
        let estimate_swapped_balance_ctx = MockCurrencySwap::estimate_swapped_balance_context();
        estimate_swapped_balance_ctx.expect().never();
        let swap_ctx = MockCurrencySwap::swap_context();
        swap_ctx.expect().never();

        // Prepare EVM call.
        let mut input = EvmDataWriter::new_with_selector(Action::Swap)
            .write(H256::from(alice.as_ref()))
            .build();
        input.extend_from_slice(&hex_literal::hex!("1000")); // bad input

        // Invoke the function under test.
        let config = <Test as pallet_evm::Config>::config();
        let execinfo = <Test as pallet_evm::Config>::Runner::call(
            alice_evm,
            *PRECOMPILE_ADDRESS,
            input,
            swap_balance.into(),
            50_123, // a reasonable upper bound for tests
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
            fp_evm::ExitReason::Error(fp_evm::ExitError::Other("junk at the end of input".into()))
        );
        assert_eq!(execinfo.used_gas.standard, expected_gas_usage.into());
        assert_eq!(execinfo.value, Vec::<u8>::new());
        assert_eq!(execinfo.logs, Vec::new());

        // Assert state changes.
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&alice_evm),
            alice_evm_balance - expected_fee
        );
        assert_eq!(<Balances as Currency<_>>::total_balance(&alice), 0);
        assert_eq!(
            <EvmBalances as Currency<_>>::total_balance(&PRECOMPILE_ADDRESS),
            0
        );

        // Assert mock invocations.
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
    });
}
