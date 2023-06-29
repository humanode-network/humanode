//! The tests for the pallet.

use frame_support::{
    assert_noop, assert_ok, assert_storage_noop, sp_runtime::DispatchError, traits::Currency,
};
use mockall::predicate;
use sp_core::H160;
use sp_std::str::FromStr;

use crate::{mock::*, *};

#[test]
fn swap_works() {
    new_test_ext().execute_with_ext(|_| {
        Balances::make_free_balance_be(&42, 1000);
        let alice_evm = H160::from_str("1000000000000000000000000000000000000001").unwrap();

        assert_eq!(Balances::total_balance(&42), 1000);
        assert_eq!(EvmBalances::total_balance(&alice_evm), 0);

        // Set mock expectations.
        let swap_ctx = MockCurrencySwap::swap_context();
        swap_ctx
            .expect()
            .once()
            .with(
                predicate::eq(&42),
                predicate::eq(alice_evm),
                predicate::eq(100),
            )
            .return_const(Ok(200));

        // Invoke the function under test.
        assert_ok!(CurrencySwap::swap(
            RuntimeOrigin::signed(42),
            alice_evm,
            100
        ));

        assert_eq!(EvmBalances::total_balance(&alice_evm), 200);
    });
}
