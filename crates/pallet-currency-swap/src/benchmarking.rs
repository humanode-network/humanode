//! The benchmarks for the pallet.

use frame_benchmarking::benchmarks;
use frame_support::{assert_ok, dispatch::DispatchResult, traits::Currency};
use frame_system::RawOrigin;

use crate::*;

const WITHDRAW_BALANCE: u32 = 100;
const DEPOSIT_BALANCE: u32 = 100;

/// The benchmarking extension for the currency swap interface.
pub trait CurrencySwap<AccountIdFrom, AccountIdTo>:
    primitives_currency_swap::CurrencySwap<AccountIdFrom, AccountIdTo>
{
    /// The data to be passed from `prepare` to `verify`.
    type Data;

    /// Prepare currency swap environment.
    fn prepare() -> Self::Data;
    /// Verify currency swap environment,
    fn verify(data: Self::Data) -> DispatchResult;
}

/// The benchmark interface into the environment.
pub trait Interface: super::Config {
    /// Obtain an `AccountIdFrom`.
    ///
    /// This is an account id balances withdrawed from.
    fn from() -> <Self as frame_system::Config>::AccountId;

    /// Obtain an `AccountIdTo`.
    ///
    /// This is an account id balances deposited to.
    fn to() -> <Self as Config>::AccountIdTo;
}

benchmarks! {
    where_clause {
        where
            T: Interface,
            <T as super::Config>::CurrencySwap: CurrencySwap<
                <T as frame_system::Config>::AccountId,
                <T as super::Config>::AccountIdTo
            >,
    }

    swap {
        let from = <T as Interface>::from();
        let to = <T as Interface>::to();
        let init_balance: u32 = 1000;

        let _ = <FromCurrencyOf<T>>::deposit_creating(&from, init_balance.into());

        let from_balance_before = <FromCurrencyOf<T>>::total_balance(&from);
        let to_balance_before = <ToCurrencyOf<T>>::total_balance(&to);

        let currency_swap = <T as super::Config>::CurrencySwap::prepare();

        let origin = RawOrigin::Signed(from.clone());

    }: _(origin, to.clone(), WITHDRAW_BALANCE.into())
    verify {
        let from_balance_after = <FromCurrencyOf<T>>::total_balance(&from);
        let to_balance_after = <ToCurrencyOf<T>>::total_balance(&to);

        assert_eq!(from_balance_before - from_balance_after, WITHDRAW_BALANCE.into());
        assert_eq!(to_balance_after - to_balance_before, DEPOSIT_BALANCE.into());

        assert_ok!(<T as super::Config>::CurrencySwap::verify(currency_swap));
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test,
    );
}

#[cfg(test)]
impl Interface for crate::mock::Test {
    fn from() -> <Self as frame_system::Config>::AccountId {
        42
    }

    fn to() -> <Self as Config>::AccountIdTo {
        use sp_std::str::FromStr;

        mock::EvmAccountId::from_str("1000000000000000000000000000000000000001").unwrap()
    }
}

#[cfg(test)]
impl CurrencySwap<mock::AccountId, mock::EvmAccountId>
    for <crate::mock::Test as super::Config>::CurrencySwap
{
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
                    DEPOSIT_BALANCE.into(),
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
}
