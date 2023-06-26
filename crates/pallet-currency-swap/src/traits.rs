//! Traits we use and expose.

use frame_support::{sp_runtime::DispatchError, traits::Currency};

pub trait CurrencySwap<AccountIdFrom, AccountIdTo> {
    type From: Currency<AccountIdFrom>;
    type To: Currency<AccountIdTo>;

    fn swap(
        account_id_from: &AccountIdFrom,
        account_id_to: &AccountIdTo,
        amount: &<Self::From as Currency<AccountIdFrom>>::Balance,
    ) -> Result<<Self::To as Currency<AccountIdTo>>::Balance, DispatchError>;
}
