//! Currency swap related primitives.

// Either generate code at stadard mode, or `no_std`, based on the `std` feature presence.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{sp_runtime::DispatchError, traits::Currency};

/// Currency swap interface.
pub trait CurrencySwap<AccountIdFrom, AccountIdTo> {
    /// The currency to convert from.
    type From: Currency<AccountIdFrom>;

    /// The currency type balances send to.
    type To: Currency<AccountIdTo>;

    /// A possible error happens during the actual swap logic.
    type Error: Into<DispatchError>;

    /// The actual swap logic.
    fn swap(
        imbalance: <Self::From as Currency<AccountIdFrom>>::NegativeImbalance,
    ) -> Result<<Self::To as Currency<AccountIdTo>>::NegativeImbalance, Self::Error>;
}
