//! Currency swap proxy related primitives.

// Either generate code at standard mode, or `no_std`, based on the `std` feature presence.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    sp_std::marker::PhantomData,
    traits::{Currency, OnUnbalanced},
};
use primitives_currency_swap::CurrencySwap;

/// A utility type alias for easy access to [`CurrencySwap::From`] of [`Config::CurrencySwap`].
type CurrencyFromFor<T> = <<T as Config>::CurrencySwap as CurrencySwap<
    <T as Config>::AccountIdFrom,
    <T as Config>::AccountIdTo,
>>::From;

/// A utility type alias for easy access to [`CurrencySwap::To`] of [`Config::CurrencySwap`].
type CurrencyToFor<T> = <<T as Config>::CurrencySwap as CurrencySwap<
    <T as Config>::AccountIdFrom,
    <T as Config>::AccountIdTo,
>>::To;

/// A utility type alias for easy access to [`Currency::NegativeImbalance`] of
/// [`CurrencySwap::From`] of [`Config::CurrencySwap`].
type CurrencyFromNegativeImbalanceFor<T> =
    <CurrencyFromFor<T> as Currency<<T as Config>::AccountIdFrom>>::NegativeImbalance;

/// A utility type alias for easy access to [`Currency::NegativeImbalance`] of
/// [`CurrencySwap::To`] of [`Config::CurrencySwap`].
type CurrencyToNegativeImbalanceFor<T> =
    <CurrencyToFor<T> as Currency<<T as Config>::AccountIdTo>>::NegativeImbalance;

/// The general config for the currency swap proxy implementations.
pub trait Config {
    /// The type used as an Account ID for the currency we proxy from.
    type AccountIdFrom;
    /// The type used as an Account ID for the currency we proxy to.
    type AccountIdTo;

    /// The currency swap implementation to use for proxying.
    type CurrencySwap: CurrencySwap<Self::AccountIdFrom, Self::AccountIdTo>;
}

/// An [`OnUnbalanced`] implementation that routes the imbalance through the currency swap and
/// passes the resulting imbalance to the `To`.
/// If swap fails, will try to pass the original imbalance to the `Fallback`.
pub struct SwapUnbalanced<T, To, Fallback>(PhantomData<(T, To, Fallback)>);

impl<T, To, Fallback> OnUnbalanced<CurrencyFromNegativeImbalanceFor<T>>
    for SwapUnbalanced<T, To, Fallback>
where
    T: Config,
    To: OnUnbalanced<CurrencyToNegativeImbalanceFor<T>>,
    Fallback: OnUnbalanced<CurrencyFromNegativeImbalanceFor<T>>,
{
    fn on_nonzero_unbalanced(amount: CurrencyFromNegativeImbalanceFor<T>) {
        let amount = match T::CurrencySwap::swap(amount) {
            Ok(amount) => amount,
            Err(primitives_currency_swap::Error {
                cause: error,
                incoming_imbalance,
            }) => {
                let error: frame_support::sp_runtime::DispatchError = error.into();
                frame_support::log::error!("unable to route the funds through the swap: {error:?}");
                Fallback::on_unbalanceds(sp_std::iter::once(incoming_imbalance));
                return;
            }
        };
        To::on_unbalanceds(sp_std::iter::once(amount))
    }
}
