//! The benchmarks for the pallet.

// Allow integer and float arithmetic in tests.
#![allow(clippy::arithmetic_side_effects, clippy::float_arithmetic)]

use frame_benchmarking::benchmarks;
use frame_support::{assert_ok, dispatch::DispatchResult, traits::Currency};
use frame_system::RawOrigin;

use crate::*;

/// The benchmark interface into the environment.
pub trait Interface: super::Config {
    /// The data to be passed from `prepare` to `verify`.
    type Data;

    /// Prepare currency swap environment.
    fn prepare() -> Self::Data;

    /// Verify currency swap environment,
    fn verify(data: Self::Data) -> DispatchResult;

    /// Obtain the Account ID the balance is swapped from.
    fn from_account_id() -> <Self as frame_system::Config>::AccountId;

    /// Obtain the amount of balance to withdraw from the swap source account.
    fn from_balance() -> FromBalanceOf<Self>;

    /// Obtain the Account ID the balance is swapped to.
    fn to_account_id() -> <Self as Config>::AccountIdTo;

    /// Obtain the amount of balance to deposit to the swap destination account.
    fn to_balance() -> ToBalanceOf<Self>;
}

benchmarks! {
    where_clause {
        where
            T: Interface,
    }

    swap {
        let from = <T as Interface>::from_account_id();
        let from_balance = <T as Interface>::from_balance();
        let to = <T as Interface>::to_account_id();
        let to_balance =  <T as Interface>::to_balance();
        let init_balance: u32 = 1000;

        let _ = <FromCurrencyOf<T>>::deposit_creating(&from, init_balance.into());

        let from_balance_before = <FromCurrencyOf<T> as Currency<_>>::total_balance(&from);
        let to_balance_before = <ToCurrencyOf<T> as Currency<_>>::total_balance(&to);

        let currency_swap = <T as Interface>::prepare();

        let origin = RawOrigin::Signed(from.clone());

    }: _(origin, to.clone(), from_balance)
    verify {
        let from_balance_after = <FromCurrencyOf<T> as Currency<_>>::total_balance(&from);
        let to_balance_after = <ToCurrencyOf<T> as Currency<_>>::total_balance(&to);

        assert_eq!(from_balance_before - from_balance_after, from_balance);
        assert_eq!(to_balance_after - to_balance_before, to_balance);

        assert_ok!(<T as Interface>::verify(currency_swap));
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test,
    );
}

#[cfg(test)]
impl Interface for crate::mock::Test {
    type Data = (
        std::sync::MutexGuard<'static, ()>,
        mock::__mock_MockCurrencySwap_CurrencySwap_9230394375286242749::__estimate_swapped_balance::Context,
        mock::__mock_MockCurrencySwap_CurrencySwap_9230394375286242749::__swap::Context,
    );

    fn prepare() -> Self::Data {
        let mock_runtime_guard = mock::runtime_lock();

        let estimate_swapped_balance_ctx =
            mock::MockCurrencySwap::estimate_swapped_balance_context();
        estimate_swapped_balance_ctx
            .expect()
            .times(1..)
            .return_const(Self::to_balance());
        let swap_ctx = mock::MockCurrencySwap::swap_context();
        swap_ctx.expect().times(1..).return_once(move |_| {
            Ok(
                <mock::EvmBalances as Currency<mock::EvmAccountId>>::NegativeImbalance::new(
                    Self::to_balance(),
                ),
            )
        });

        (mock_runtime_guard, estimate_swapped_balance_ctx, swap_ctx)
    }

    fn verify(data: Self::Data) -> DispatchResult {
        let (mock_runtime_guard, estimate_swapped_balance_ctx, swap_ctx) = data;
        estimate_swapped_balance_ctx.checkpoint();
        swap_ctx.checkpoint();
        drop(mock_runtime_guard);
        Ok(())
    }

    fn from_account_id() -> <Self as frame_system::Config>::AccountId {
        42
    }

    fn from_balance() -> FromBalanceOf<Self> {
        100
    }

    fn to_account_id() -> <Self as Config>::AccountIdTo {
        use sp_std::str::FromStr;

        mock::EvmAccountId::from_str("1000000000000000000000000000000000000001").unwrap()
    }

    fn to_balance() -> ToBalanceOf<Self> {
        100
    }
}
