#![allow(clippy::integer_arithmetic)] // not a problem in tests

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
        let alice_balance = 100 * 10u128.pow(18);
        let swap_balance = 10 * 10u128.pow(18);

        let expected_gas_usage: u64 = 21216 + 200;
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_balance);

        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice_evm), alice_balance);
        assert_eq!(Balances::total_balance(&alice), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Set mock expectations.
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
            config,
        )
        .unwrap();
        assert_eq!(
            execinfo.exit_reason,
            fp_evm::ExitReason::Succeed(fp_evm::ExitSucceed::Returned)
        );
        assert_eq!(execinfo.used_gas, expected_gas_usage.into());
        assert_eq!(execinfo.value, EvmDataWriter::new().write(true).build());
        assert_eq!(execinfo.logs, Vec::new());

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&alice_evm),
            alice_balance - swap_balance - expected_fee
        );
        assert_eq!(Balances::total_balance(&alice), swap_balance);

        // Assert mock invocations.
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

        let alice_balance = 100 * 10u128.pow(18);
        let swap_balance = 100 * 10u128.pow(18) - expected_fee - 1;

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_balance);

        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice_evm), alice_balance);
        assert_eq!(Balances::total_balance(&alice), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Set mock expectations.
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
            config,
        )
        .unwrap();
        assert_eq!(
            execinfo.exit_reason,
            fp_evm::ExitReason::Succeed(fp_evm::ExitSucceed::Returned)
        );
        assert_eq!(execinfo.used_gas, expected_gas_usage.into());
        assert_eq!(execinfo.value, EvmDataWriter::new().write(true).build());
        assert_eq!(execinfo.logs, Vec::new());

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&alice_evm),
            alice_balance - swap_balance - expected_fee
        );
        assert_eq!(EvmBalances::total_balance(&alice_evm), 1);
        assert_eq!(Balances::total_balance(&alice), swap_balance);

        // Assert mock invocations.
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
        let alice_balance = 100 * 10u128.pow(18);
        let swap_balance = 1000 * 10u128.pow(18); // more than we have

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_balance);

        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice_evm), alice_balance);
        assert_eq!(Balances::total_balance(&alice), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Set mock expectations.
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
        assert_eq!(EvmBalances::total_balance(&alice_evm), alice_balance);
        assert_eq!(Balances::total_balance(&alice), 0);

        // Assert mock invocations.
        swap_ctx.checkpoint();
    });
}

/// This test verifies that the swap precompile call behaves as expected when the currency swap
/// implementation fails.
/// The fee is consumed, but the value is not.
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
        let alice_balance = 100 * 10u128.pow(18);
        let swap_balance = 10 * 10u128.pow(18);

        let expected_gas_usage: u64 = 50_123; // all the allowed fee will be consumed
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_balance);

        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice_evm), alice_balance);
        assert_eq!(Balances::total_balance(&alice), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Set mock expectations.
        let swap_ctx = MockCurrencySwap::swap_context();
        swap_ctx
            .expect()
            .once()
            .with(predicate::eq(
                <EvmBalances as Currency<EvmAccountId>>::NegativeImbalance::new(swap_balance),
            ))
            .return_once(move |_| Err(sp_runtime::DispatchError::Other("test")));

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
            config,
        )
        .unwrap();
        assert_eq!(
            execinfo.exit_reason,
            fp_evm::ExitReason::Error(fp_evm::ExitError::Other("unable to swap funds".into()))
        );
        assert_eq!(execinfo.used_gas, expected_gas_usage.into());
        assert_eq!(execinfo.value, Vec::<u8>::new());
        assert_eq!(execinfo.logs, Vec::new());

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&alice_evm),
            alice_balance - expected_fee
        );
        assert_eq!(Balances::total_balance(&alice), 0);

        // Assert mock invocations.
        swap_ctx.checkpoint();
    });
}

/// This test verifies that the swap precompile call is unable to swap the whole account balance.
/// This is not so much the desired behaviour, but rather an undesired effect of the current
/// implementation that is nonetheless specified and verified with tests.
/// The precense of this test ensures that when/if this behaviour is fixed, this test will start
/// failing and will have to be replaced with another test that verified the new behaviour.
/// See also: [`swap_works_almost_full_balance`].
#[test]
fn swap_fail_full_balance() {
    new_test_ext().execute_with_ext(|_| {
        let alice_evm = H160::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ));
        let alice = AccountId::from(hex_literal::hex!(
            "1000000000000000000000000000000000000000000000000000000000000001"
        ));

        let expected_gas_usage: u64 = 21216 + 200;
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        let alice_balance = 100 * 10u128.pow(18);
        let swap_balance = 100 * 10u128.pow(18) - expected_fee;

        // Prepare the test state.
        EvmBalances::make_free_balance_be(&alice_evm, alice_balance);

        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice_evm), alice_balance);
        assert_eq!(Balances::total_balance(&alice), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Set mock expectations.
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
            expected_gas_usage, // the exact amount of fee we'll be using
            Some(*GAS_PRICE),
            Some(*GAS_PRICE),
            None,
            Vec::new(),
            true,
            true,
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
        assert_eq!(EvmBalances::total_balance(&alice_evm), alice_balance);
        assert_eq!(Balances::total_balance(&alice), 0);

        // Assert mock invocations.
        swap_ctx.checkpoint();
    });
}
