use fp_evm::{ExitReason, ExitSucceed};
use frame_support::assert_ok;
use sp_core::Get;

use crate::{mock::*, *};

/// This test verifies that the swap call works in the happy path.
#[test]
fn swap_works() {
    new_test_ext().execute_with_ext(|_| {
        let swap_evm_account = EvmAccountId::from(hex_literal::hex!(
            "7700000000000000000000000000000000000077"
        ));
        let swap_balance = 100;

        // Check test preconditions.
        assert_eq!(Balances::total_balance(&alice()), INIT_BALANCE);
        assert_eq!(EvmBalances::total_balance(&swap_evm_account), 0);

        // We should remember expected evm transaction hash before execution as nonce is increased
        // after the execution.
        let expected_evm_transaction_hash = ethereum_transfer_transaction::<Test>(
            BridgePotEvm::get(),
            swap_evm_account,
            swap_balance,
        )
        .hash();

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_ok!(EvmSwap::swap(
            RuntimeOrigin::signed(alice()),
            swap_evm_account,
            swap_balance
        ));

        // Assert state changes.
        assert_eq!(
            Balances::total_balance(&alice()),
            INIT_BALANCE - swap_balance
        );
        assert_eq!(
            Balances::total_balance(&BridgePotNative::get()),
            BRIDGE_INIT_BALANCE + swap_balance
        );
        assert_eq!(EvmBalances::total_balance(&swap_evm_account), swap_balance);
        assert_eq!(
            EvmBalances::total_balance(&BridgePotEvm::get()),
            BRIDGE_INIT_BALANCE - swap_balance
        );
        assert_eq!(EvmBalances::total_balance(&*PRECOMPILE_ADDRESS), 0);
        System::assert_has_event(RuntimeEvent::EvmSwap(Event::BalancesSwapped {
            from: alice(),
            withdrawed_amount: swap_balance,
            to: swap_evm_account,
            deposited_amount: swap_balance,
            evm_transaction_hash: expected_evm_transaction_hash,
        }));
        System::assert_has_event(RuntimeEvent::Ethereum(pallet_ethereum::Event::Executed {
            from: BridgePotEvm::get(),
            to: swap_evm_account,
            transaction_hash: expected_evm_transaction_hash,
            exit_reason: ExitReason::Succeed(ExitSucceed::Stopped),
            extra_data: vec![],
        }));
    });
}
