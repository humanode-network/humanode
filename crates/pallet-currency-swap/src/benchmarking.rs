//! The benchmarks for the pallet.

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

    /// Obtain an account id balances withdrawed from.
    fn from() -> <Self as frame_system::Config>::AccountId;

    /// Obtain withdraw balances amount.
    fn withdraw_amount() -> FromBalanceOf<Self>;

    /// Obtain an account id balances deposited to.
    fn to() -> <Self as Config>::AccountIdTo;

    /// Obtain deposit balances amount.
    fn deposit_amount() -> ToBalanceOf<Self>;
}

benchmarks! {
    where_clause {
        where
            T: Interface,
    }

    swap {
        let from = <T as Interface>::from();
        let withdraw_amount = <T as Interface>::withdraw_amount();
        let to = <T as Interface>::to();
        let deposit_amount =  <T as Interface>::deposit_amount();
        let init_balance: u32 = 1000;

        let _ = <FromCurrencyOf<T>>::deposit_creating(&from, init_balance.into());

        let from_balance_before = <FromCurrencyOf<T>>::total_balance(&from);
        let to_balance_before = <ToCurrencyOf<T>>::total_balance(&to);

        let currency_swap = <T as Interface>::prepare();

        let origin = RawOrigin::Signed(from.clone());

    }: _(origin, to.clone(), withdraw_amount)
    verify {
        let from_balance_after = <FromCurrencyOf<T>>::total_balance(&from);
        let to_balance_after = <ToCurrencyOf<T>>::total_balance(&to);

        assert_eq!(from_balance_before - from_balance_after, withdraw_amount);
        assert_eq!(to_balance_after - to_balance_before, deposit_amount);

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
        mock::__mock_MockCurrencySwap_CurrencySwap_9230394375286242749::__swap::Context,
    );

    fn prepare() -> Self::Data {
        let mock_runtime_guard = mock::runtime_lock();

        let swap_ctx = mock::MockCurrencySwap::swap_context();
        swap_ctx.expect().times(1..).return_once(move |_| {
            Ok(
                <mock::EvmBalances as Currency<mock::EvmAccountId>>::NegativeImbalance::new(
                    Self::deposit_amount().into(),
                ),
            )
        });

        (mock_runtime_guard, swap_ctx)
    }

    fn verify(data: Self::Data) -> DispatchResult {
        let (mock_runtime_guard, swap_ctx) = data;
        swap_ctx.checkpoint();
        drop(mock_runtime_guard);
        Ok(())
    }

    fn from() -> <Self as frame_system::Config>::AccountId {
        42
    }

    fn withdraw_amount() -> FromBalanceOf<Self> {
        100
    }

    fn to() -> <Self as Config>::AccountIdTo {
        use sp_std::str::FromStr;

        mock::EvmAccountId::from_str("1000000000000000000000000000000000000001").unwrap()
    }

    fn deposit_amount() -> ToBalanceOf<Self> {
        100
    }
}
