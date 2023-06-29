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
        account_id_from: &AccountIdFrom,
        account_id_to: &AccountIdTo,
        amount: <Self::From as Currency<AccountIdFrom>>::Balance,
    ) -> Result<<Self::To as Currency<AccountIdTo>>::Balance, Self::Error>;
}
