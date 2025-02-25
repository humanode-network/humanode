use pallet_evm::{Config, Runner};
use precompile_utils::{EvmDataWriter, LogsBuilder};
use sp_core::H256;

use crate::{mock::*, *};

/// A test utility that performs gas to fee computation.
/// Might not be explicitly correct, but does the job.
fn gas_to_fee(gas: u64) -> Balance {
    u128::from(gas) * u128::try_from(*GAS_PRICE).unwrap()
}

/// This test verifies that the swap precompile call works in the happy path.
#[test]
fn swap_works() {
    new_test_ext().execute_with_ext(|_| {
        let swap_native_account = AccountId::from(hex_literal::hex!(
            "7700000000000000000000000000000000000000000000000000000000000077"
        ));
        let swap_balance = 100;

        let expected_gas_usage: u64 = 21216 + 200;
        let expected_fee: Balance = gas_to_fee(expected_gas_usage);

        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice_evm()), INIT_BALANCE);
        assert_eq!(Balances::total_balance(&swap_native_account), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Prepare EVM call.
        let input = EvmDataWriter::new_with_selector(precompile::Action::Swap)
            .write(H256::from(swap_native_account.as_ref()))
            .build();

        // Invoke the function under test.
        let execinfo = <Test as pallet_evm::Config>::Runner::call(
            alice_evm(),
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
            Test::config(),
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
                precompile::SELECTOR_LOG_SWAP,
                alice_evm(),
                H256::from(swap_native_account.as_ref()),
                EvmDataWriter::new().write(swap_balance).build(),
            )]
        );

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&alice_evm()),
            INIT_BALANCE - swap_balance - expected_fee
        );
        assert_eq!(
            EvmBalances::total_balance(&BridgePotEvm::get()),
            INIT_BALANCE + swap_balance,
        );
        assert_eq!(Balances::total_balance(&swap_native_account), swap_balance);
        assert_eq!(
            Balances::total_balance(&BridgePotNative::get()),
            INIT_BALANCE - swap_balance
        );
    });
}
