use sp_core::Get;

use crate::{mock::*, *};

mod evm_to_native;
mod native_to_evm;

/// This test verifies that basic genesis setup works in the happy path.
#[test]
fn basic_setup_works() {
    new_test_ext().execute_with_ext(|_| {
        assert_eq!(
            Balances::total_balance(&BridgePotNative::get()),
            INIT_BALANCE
        );
        assert_eq!(Balances::total_balance(&alice()), INIT_BALANCE);
        assert_eq!(Balances::total_balance(&bob()), INIT_BALANCE);
        assert_eq!(
            EvmBalances::total_balance(&BridgePotEvm::get()),
            INIT_BALANCE
        );
        assert_eq!(EvmBalances::total_balance(&alice_evm()), INIT_BALANCE);
        assert_eq!(EvmBalances::total_balance(&bob_evm()), INIT_BALANCE);

        assert_eq!(Balances::total_issuance(), EvmBalances::total_issuance());
    });
}
