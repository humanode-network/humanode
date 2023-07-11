//! Currency (as in [`fungible::Balanced`]) swap related primitives.

// Either generate code at stadard mode, or `no_std`, based on the `std` feature presence.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{sp_runtime::DispatchError, traits::fungible};

/// Currency swap interface.
pub trait CurrencySwap<AccountIdFrom, AccountIdTo> {
    /// The currency to convert from.
    type From: fungible::Balanced<AccountIdFrom>;

    /// The currency to convert to.
    type To: fungible::Balanced<AccountIdTo>;

    /// A possible error happens during the actual swap logic.
    type Error: Into<DispatchError>;

    /// The actual swap logic.
    fn swap(
        credit: FromCreditOf<Self, AccountIdFrom, AccountIdTo>,
    ) -> Result<
        ToCreditOf<Self, AccountIdFrom, AccountIdTo>,
        ErrorFor<Self, AccountIdFrom, AccountIdTo>,
    >;
}

/// An easy way to access the [`fungible::CreditOf`] of [`CurrencySwap::From`] of `T`.
pub type FromCreditOf<T, AccountIdFrom, AccountIdTo> =
    fungible::CreditOf<AccountIdFrom, <T as CurrencySwap<AccountIdFrom, AccountIdTo>>::From>;

/// An easy way to access the [`fungible::CreditOf`] of [`CurrencySwap::To`] of `T`.
pub type ToCreditOf<T, AccountIdFrom, AccountIdTo> =
    fungible::CreditOf<AccountIdTo, <T as CurrencySwap<AccountIdFrom, AccountIdTo>>::To>;

/// A type alias for compact declaration of the error type for the [`CurrencySwap::swap`] call.
pub type ErrorFor<T, AccountIdFrom, AccountIdTo> = Error<
    FromCreditOf<T, AccountIdFrom, AccountIdTo>,
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
