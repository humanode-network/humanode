use fp_ethereum::ValidatedTransaction;
use fp_evm::{ExitReason, ExitSucceed};
use frame_support::{
    dispatch::{Pays, PostDispatchInfo},
    traits::OnFinalize,
};
use pallet_evm::GasWeightMapping;
use precompile_utils::{EvmDataWriter, LogsBuilder};
use sp_core::H256;

use crate::{mock::*, *};

/// This test verifies that the swap precompile call works in the happy path.
#[test]
fn swap_works() {
    new_test_ext().execute_with_ext(|_| {
        let swap_native_account = AccountId::from(hex_literal::hex!(
            "7700000000000000000000000000000000000000000000000000000000000077"
        ));
        let swap_balance = 100;

        let expected_gas_usage: u64 = 21216 + 200;

        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice_evm()), INIT_BALANCE);
        assert_eq!(Balances::total_balance(&swap_native_account), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Prepare EVM call.
        let transaction = pallet_ethereum::Transaction::EIP1559(ethereum::EIP1559Transaction {
            chain_id: <Test as pallet_evm::Config>::ChainId::get(),
            nonce: pallet_evm::Pallet::<Test>::account_basic(&alice_evm())
                .0
                .nonce,
            max_priority_fee_per_gas: 0.into(),
            max_fee_per_gas: 0.into(),
            gas_limit: 50_000.into(), // a reasonable upper bound for tests
            action: ethereum::TransactionAction::Call(*PRECOMPILE_ADDRESS),
            value: U256::from(swap_balance),
            input: EvmDataWriter::new_with_selector(precompile::Action::Swap)
                .write(H256::from(swap_native_account.as_ref()))
                .build(),
            access_list: Default::default(),
            odd_y_parity: false,
            r: Default::default(),
            s: Default::default(),
        });
        let transaction_hash = transaction.hash();

        // Invoke the function under test.
        let post_info =
            pallet_ethereum::ValidatedTransaction::<Test>::apply(alice_evm(), transaction).unwrap();

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
        assert_eq!(
            EvmBalances::total_balance(&alice_evm()),
            INIT_BALANCE - swap_balance
        );
        assert_eq!(
            EvmBalances::total_balance(&BridgePotEvm::get()),
            BRIDGE_INIT_BALANCE + swap_balance,
        );
        assert_eq!(Balances::total_balance(&swap_native_account), swap_balance);
        assert_eq!(
            Balances::total_balance(&BridgePotNative::get()),
            BRIDGE_INIT_BALANCE - swap_balance
        );
        System::assert_has_event(RuntimeEvent::Ethereum(pallet_ethereum::Event::Executed {
            from: alice_evm(),
            to: *PRECOMPILE_ADDRESS,
            transaction_hash,
            exit_reason: ExitReason::Succeed(ExitSucceed::Returned),
            extra_data: vec![],
        }));

        // Finalize block to check transaction logs.
        Ethereum::on_finalize(1);

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
                alice_evm(),
                H256::from(swap_native_account.as_ref()),
                EvmDataWriter::new().write(swap_balance).build(),
            )]
        );
    });
}

/// This test verifies that the swap precompile call works when we transfer *almost* the full
/// account balance.
/// Almost because we leave one token left on the source account.
#[test]
fn swap_works_almost_full_balance() {
    new_test_ext().execute_with_ext(|_| {
        let swap_native_account = AccountId::from(hex_literal::hex!(
            "7700000000000000000000000000000000000000000000000000000000000077"
        ));

        let expected_gas_usage: u64 = 21216 + 200;
        let swap_balance = INIT_BALANCE - 1;

        // Check test preconditions.
        assert_eq!(<EvmBalances>::total_balance(&alice_evm()), INIT_BALANCE);
        assert_eq!(<Balances>::total_balance(&swap_native_account), 0);

        // Set block number to enable events.
        System::set_block_number(1);

        // Prepare EVM call.
        let transaction = pallet_ethereum::Transaction::EIP1559(ethereum::EIP1559Transaction {
            chain_id: <Test as pallet_evm::Config>::ChainId::get(),
            nonce: pallet_evm::Pallet::<Test>::account_basic(&alice_evm())
                .0
                .nonce,
            max_priority_fee_per_gas: 0.into(),
            max_fee_per_gas: 0.into(),
            gas_limit: 50_000.into(), // a reasonable upper bound for tests
            action: ethereum::TransactionAction::Call(*PRECOMPILE_ADDRESS),
            value: U256::from(swap_balance),
            input: EvmDataWriter::new_with_selector(precompile::Action::Swap)
                .write(H256::from(swap_native_account.as_ref()))
                .build(),
            access_list: Default::default(),
            odd_y_parity: false,
            r: Default::default(),
            s: Default::default(),
        });
        let transaction_hash = transaction.hash();

        // Invoke the function under test.
        let post_info =
            pallet_ethereum::ValidatedTransaction::<Test>::apply(alice_evm(), transaction).unwrap();

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
        assert_eq!(
            EvmBalances::total_balance(&alice_evm()),
            INIT_BALANCE - swap_balance
        );
        assert_eq!(
            EvmBalances::total_balance(&BridgePotEvm::get()),
            BRIDGE_INIT_BALANCE + swap_balance,
        );
        assert_eq!(Balances::total_balance(&swap_native_account), swap_balance);
        assert_eq!(
            Balances::total_balance(&BridgePotNative::get()),
            BRIDGE_INIT_BALANCE - swap_balance
        );
        System::assert_has_event(RuntimeEvent::Ethereum(pallet_ethereum::Event::Executed {
            from: alice_evm(),
            to: *PRECOMPILE_ADDRESS,
            transaction_hash,
            exit_reason: ExitReason::Succeed(ExitSucceed::Returned),
            extra_data: vec![],
        }));

        // Finalize block to check transaction logs.
        Ethereum::on_finalize(1);

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
                alice_evm(),
                H256::from(swap_native_account.as_ref()),
                EvmDataWriter::new().write(swap_balance).build(),
            )]
        );
    });
}
