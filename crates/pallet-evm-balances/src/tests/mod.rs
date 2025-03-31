//! The tests for the pallet.

use frame_support::{assert_ok, traits::Currency, weights::Weight};
use pallet_evm::{FeeCalculator, FixedGasWeightMapping, GasWeightMapping, Runner};
use sp_core::{H160, U256};
use sp_runtime::traits::UniqueSaturatedInto;
use sp_std::str::FromStr;

use crate::{mock::*, *};

mod currency;
mod fungible;
mod fungible_conformance_tests;

fn assert_total_issuance_invariant() {
    let iterated_total_issuance: u64 = <pallet_evm_system::Account<Test>>::iter_values()
        .map(|account_data| account_data.data.total())
        .sum();

    let total_issuance = EvmBalances::total_issuance();

    assert_eq!(iterated_total_issuance, total_issuance);
}

#[test]
fn basic_setup_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check the accounts.
        assert_eq!(
            <EvmBalances>::account(&alice()),
            account_data::AccountData { free: INIT_BALANCE }
        );
        assert_eq!(
            <EvmBalances>::account(&bob()),
            account_data::AccountData { free: INIT_BALANCE }
        );

        assert_total_issuance_invariant();
    });
}

#[test]
fn evm_system_removing_account_non_zero_balance() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test preconditions.
        let contract = H160::from_str("1000000000000000000000000000000000000003").unwrap();
        EVM::create_account(contract, vec![1, 2, 3]);

        // Transfer some balance to contract address.
        assert_ok!(EvmBalances::transfer(
            &alice(),
            &contract,
            1000,
            ExistenceRequirement::KeepAlive
        ));

        assert_eq!(EvmBalances::free_balance(contract), 1000);

        // Invoke the function under test.
        EVM::remove_account(&contract);

        // Assert state changes.
        assert_eq!(EvmBalances::free_balance(contract), 1000);
        assert!(EvmSystem::account_exists(&contract));

        assert_total_issuance_invariant();
    });
}

#[test]
fn evm_fee_deduction() {
    new_test_ext().execute_with_ext(|_| {
		let charlie = H160::from_str("1000000000000000000000000000000000000003").unwrap();

		// Seed account
		let _ = <Test as pallet_evm::Config>::Currency::deposit_creating(&charlie, 100);
		assert_eq!(EvmBalances::free_balance(charlie), 100);

		// Deduct fees as 10 units
		let imbalance =
			<<Test as pallet_evm::Config>::OnChargeTransaction as pallet_evm::OnChargeEVMTransaction<Test>>::withdraw_fee(
				&charlie,
				U256::from(10),
			)
			.unwrap();
		assert_eq!(EvmBalances::free_balance(charlie), 90);

		// Refund fees as 5 units
		<<Test as pallet_evm::Config>::OnChargeTransaction as pallet_evm::OnChargeEVMTransaction<Test>>::correct_and_deposit_fee(&charlie, U256::from(5), U256::from(5), imbalance);
		assert_eq!(EvmBalances::free_balance(charlie), 95);

		assert_total_issuance_invariant();
	});
}

#[test]
fn evm_issuance_after_tip() {
    new_test_ext().execute_with_ext(|_| {
        let before_tip = <Test as pallet_evm::Config>::Currency::total_issuance();

        let gas_limit: u64 = 1_000_000;
        let weight_limit = FixedGasWeightMapping::<Test>::gas_to_weight(gas_limit, true);

        assert_ok!(<Test as pallet_evm::Config>::Runner::call(
            alice(),
            bob(),
            Vec::new(),
            U256::from(1),
            gas_limit,
            Some(U256::from(2_000_000_000)),
            None,
            None,
            Vec::new(),
            true,
            true,
            Some(weight_limit),
            Some(0),
            <Test as pallet_evm::Config>::config(),
        ));

        // Only base fee is burned
        let base_fee: u64 = <Test as pallet_evm::Config>::FeeCalculator::min_gas_price()
            .0
            .unique_saturated_into();

        let after_tip = <Test as pallet_evm::Config>::Currency::total_issuance();

        assert_eq!(after_tip, (before_tip - (base_fee * 21_000)));

        assert_total_issuance_invariant();
    });
}

#[test]
fn evm_refunds_should_work() {
    new_test_ext().execute_with_ext(|_| {
        let before_call = EVM::account_basic(&alice()).0.balance;
        // Gas price is not part of the actual fee calculations anymore, only the base fee.
        //
        // Because we first deduct max_fee_per_gas * gas_limit (2_000_000_000 * 1000000) we need
        // to ensure that the difference (max fee VS base fee) is refunded.

        let gas_limit: u64 = 1_000_000;
        let weight_limit = FixedGasWeightMapping::<Test>::gas_to_weight(gas_limit, true);

        let _ = <Test as pallet_evm::Config>::Runner::call(
            alice(),
            bob(),
            Vec::new(),
            U256::from(1),
            gas_limit,
            Some(U256::from(2_000_000_000)),
            None,
            None,
            Vec::new(),
            true,
            true,
            Some(weight_limit),
            Some(0),
            <Test as pallet_evm::Config>::config(),
        );

        let (base_fee, _) = <Test as pallet_evm::Config>::FeeCalculator::min_gas_price();
        let total_cost = (U256::from(21_000) * base_fee) + U256::from(1);
        let after_call = EVM::account_basic(&alice()).0.balance;
        assert_eq!(after_call, before_call - total_cost);

        assert_total_issuance_invariant();
    });
}

#[test]
fn evm_refunds_and_priority_should_work() {
    new_test_ext().execute_with_ext(|_| {
        let before_call = EVM::account_basic(&alice()).0.balance;
        // We deliberately set a base fee + max tip > max fee.
        // The effective priority tip will be 1GWEI instead 1.5GWEI:
        // 		(max_fee_per_gas - base_fee).min(max_priority_fee)
        //		(2 - 1).min(1.5)
        let tip = U256::from(1_500_000_000);
        let max_fee_per_gas = U256::from(2_000_000_000);
        let used_gas = U256::from(21_000);

        let gas_limit: u64 = 1_000_000;
        let weight_limit = FixedGasWeightMapping::<Test>::gas_to_weight(gas_limit, true);

        let _ = <Test as pallet_evm::Config>::Runner::call(
            alice(),
            bob(),
            Vec::new(),
            U256::from(1),
            gas_limit,
            Some(max_fee_per_gas),
            Some(tip),
            None,
            Vec::new(),
            true,
            true,
            Some(weight_limit),
            Some(0),
            <Test as pallet_evm::Config>::config(),
        );

        let (base_fee, _) = <Test as pallet_evm::Config>::FeeCalculator::min_gas_price();
        let actual_tip = (max_fee_per_gas - base_fee).min(tip) * used_gas;
        let total_cost = (used_gas * base_fee) + actual_tip + U256::from(1);
        let after_call = EVM::account_basic(&alice()).0.balance;
        // The tip is deducted but never refunded to the caller.
        assert_eq!(after_call, before_call - total_cost);

        assert_total_issuance_invariant();
    });
}

#[test]
fn evm_call_should_fail_with_priority_greater_than_max_fee() {
    new_test_ext().execute_with_ext(|_| {
        // Max priority greater than max fee should fail.
        let tip: u128 = 1_100_000_000;

        let gas_limit: u64 = 1_000_000;
        let weight_limit = FixedGasWeightMapping::<Test>::gas_to_weight(gas_limit, true);

        let result = <Test as pallet_evm::Config>::Runner::call(
            alice(),
            bob(),
            Vec::new(),
            U256::from(1),
            gas_limit,
            Some(U256::from(1_000_000_000)),
            Some(U256::from(tip)),
            None,
            Vec::new(),
            true,
            true,
            Some(weight_limit),
            Some(0),
            <Test as pallet_evm::Config>::config(),
        );
        assert!(result.is_err());
        // Some used weight is returned as part of the error.
        assert_eq!(result.unwrap_err().weight, Weight::from_parts(7, 0));

        assert_total_issuance_invariant();
    });
}

#[test]
fn evm_call_should_succeed_with_priority_equal_to_max_fee() {
    new_test_ext().execute_with_ext(|_| {
        let tip: u128 = 1_000_000_000;

        let gas_limit: u64 = 1_000_000;
        let weight_limit = FixedGasWeightMapping::<Test>::gas_to_weight(gas_limit, true);

        // Mimics the input for pre-eip-1559 transaction types where `gas_price`
        // is used for both `max_fee_per_gas` and `max_priority_fee_per_gas`.
        let result = <Test as pallet_evm::Config>::Runner::call(
            alice(),
            bob(),
            Vec::new(),
            U256::from(1),
            gas_limit,
            Some(U256::from(1_000_000_000)),
            Some(U256::from(tip)),
            None,
            Vec::new(),
            true,
            true,
            Some(weight_limit),
            Some(0),
            <Test as pallet_evm::Config>::config(),
        );
        assert!(result.is_ok());

        assert_total_issuance_invariant();
    });
}
