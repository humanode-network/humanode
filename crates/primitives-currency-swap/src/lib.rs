//! Currency swap related primitives.

// Either generate code at standard mode, or `no_std`, based on the `std` feature presence.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    sp_runtime::DispatchError,
    traits::{fungible::Inspect, Currency},
};

/// Currency swap interface.
pub trait CurrencySwap<AccountIdFrom, AccountIdTo> {
    /// The currency to convert from.
    type From: Currency<AccountIdFrom>
        + Inspect<AccountIdFrom, Balance = <Self::From as Currency<AccountIdFrom>>::Balance>;

    /// The currency to convert to.
    type To: Currency<AccountIdTo>
        + Inspect<AccountIdTo, Balance = <Self::To as Currency<AccountIdTo>>::Balance>;

    /// A possible error happens during the actual swap logic.
    type Error: Into<DispatchError>;

    /// The actual swap logic.
    fn swap(
        imbalance: FromNegativeImbalanceFor<Self, AccountIdFrom, AccountIdTo>,
    ) -> Result<
        ToNegativeImbalanceFor<Self, AccountIdFrom, AccountIdTo>,
        ErrorFor<Self, AccountIdFrom, AccountIdTo>,
    >;

    /// Obtain the estimated resulted balance value.
    fn estimate_swapped_balance(
        balance: FromBalanceFor<Self, AccountIdFrom, AccountIdTo>,
    ) -> ToBalanceFor<Self, AccountIdFrom, AccountIdTo>;
}

/// An easy way to access the [`Currency::Balance`] of [`CurrencySwap::From`] of `T`.
pub type FromBalanceFor<T, AccountIdFrom, AccountIdTo> =
    <<T as CurrencySwap<AccountIdFrom, AccountIdTo>>::From as Currency<AccountIdFrom>>::Balance;

/// An easy way to access the [`Currency::NegativeImbalance`] of [`CurrencySwap::From`] of `T`.
pub type FromNegativeImbalanceFor<T, AccountIdFrom, AccountIdTo> = <<T as CurrencySwap<
    AccountIdFrom,
    AccountIdTo,
>>::From as Currency<AccountIdFrom>>::NegativeImbalance;

/// An easy way to access the [`Currency::Balance`] of [`CurrencySwap::To`] of `T`.
pub type ToBalanceFor<T, AccountIdFrom, AccountIdTo> =
    <<T as CurrencySwap<AccountIdFrom, AccountIdTo>>::To as Currency<AccountIdTo>>::Balance;

/// An easy way to access the [`Currency::NegativeImbalance`] of [`CurrencySwap::To`] of `T`.
pub type ToNegativeImbalanceFor<T, AccountIdFrom, AccountIdTo> = <<T as CurrencySwap<
    AccountIdFrom,
    AccountIdTo,
>>::To as Currency<AccountIdTo>>::NegativeImbalance;

/// A type alias for compact declaration of the error type for the [`CurrencySwap::swap`] call.
pub type ErrorFor<T, AccountIdFrom, AccountIdTo> = Error<
    FromNegativeImbalanceFor<T, AccountIdFrom, AccountIdTo>,
    <T as CurrencySwap<AccountIdFrom, AccountIdTo>>::Error,
>;

/// An error that can occur while doing a currency swap.
#[derive(Debug)]
pub struct Error<I, E> {
    /// The underlying cause of this error.
    pub cause: E,
    /// The original imbalance that was passed to the swap operation.
    pub incoming_imbalance: I,
}
