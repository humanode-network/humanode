//! Traits we use and expose.

use frame_support::{sp_runtime::DispatchError, traits::Currency};

/// Currency swap interface.
pub trait CurrencySwap<AccountIdFrom, AccountIdTo> {
    /// The currency type balances send from.
    type From: Currency<AccountIdFrom>;
    /// The currency type balances send to.
    type To: Currency<AccountIdTo>;
    /// An error happens during the actual balances swap.
    type Error: Into<DispatchError>;

    /// Swap balances.
    fn swap(
        imbalance: <Self::From as Currency<AccountIdFrom>>::NegativeImbalance,
    ) -> Result<<Self::To as Currency<AccountIdTo>>::NegativeImbalance, Self::Error>;
}
