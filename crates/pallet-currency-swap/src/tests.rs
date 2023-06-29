//! The tests for the pallet.

use frame_support::{
    assert_noop, assert_ok, assert_storage_noop, sp_runtime::DispatchError, traits::Currency,
};
use sp_core::H160;
use sp_std::str::FromStr;

use crate::{mock::*, *};

#[test]
fn swap_works() {
    new_test_ext().execute_with_ext(|_| {
        Balances::make_free_balance_be(&42, 1000);
        let alice_evm = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let balances_before = Balances::total_issuance();
        let evm_balances_before = EvmBalances::total_issuance();

        assert_eq!(Balances::total_balance(&42), 1000);
        assert_eq!(EvmBalances::total_balance(&alice_evm), 0);

        // Invoke the function under test.
        assert_ok!(CurrencySwap::swap(
            RuntimeOrigin::signed(42),
            alice_evm,
            100
        ));

        assert_eq!(Balances::total_balance(&42), 900);
        assert_eq!(EvmBalances::total_balance(&alice_evm), 100);

        assert_eq!(Balances::total_issuance(), balances_before - 100);
        assert_eq!(EvmBalances::total_issuance(), evm_balances_before + 100);
    });
}
