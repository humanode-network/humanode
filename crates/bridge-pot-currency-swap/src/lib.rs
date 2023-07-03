//! Bridge pot currency swap implementation.

// Either generate code at stadard mode, or `no_std`, based on the `std` feature presence.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    sp_runtime::traits::Convert,
    sp_std::marker::PhantomData,
    traits::{Currency, Get},
};

pub mod existence_optional;
pub mod existence_required;

pub use existence_optional::Marker as ExistenceOptional;
pub use existence_required::Marker as ExistenceRequired;

/// The config for the generic bridge pot currency swap logic.
pub trait Config {
    /// The type representing the account key for the currency to swap from.
    type AccountIdFrom;

    /// The type representing the account key for the currency to swap to.
    type AccountIdTo;

    /// The currency to swap from.
    type CurrencyFrom: Currency<Self::AccountIdFrom>;

    /// The currency to swap to.
    type CurrencyTo: Currency<Self::AccountIdTo>;

    /// The converter to determine how the balance amount should be converted from one currency to
    /// another.
    type BalanceConverter: Convert<
        <Self::CurrencyFrom as Currency<Self::AccountIdFrom>>::Balance,
        <Self::CurrencyTo as Currency<Self::AccountIdTo>>::Balance,
    >;

    /// The account to land the balances to when receiving the funds as part of the swap operation.
    type PotFrom: Get<Self::AccountIdFrom>;

    /// The account to take the balances from when sending the funds as part of the swap operation.
    type PotTo: Get<Self::AccountIdTo>;
}

/// A [`primitives_currency_swap::CurrencySwap`] implementation that does the swap using two
/// "pot" accounts for each of end swapped currencies.
pub struct CurrencySwap<T: Config, ExistenceRequirement>(PhantomData<(T, ExistenceRequirement)>);
