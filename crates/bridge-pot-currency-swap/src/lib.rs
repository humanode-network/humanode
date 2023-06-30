//! Bridge pot currency swap implementation.

// Either generate code at stadard mode, or `no_std`, based on the `std` feature presence.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{traits::Currency, sp_runtime::DispatchError};
use primitives_currency_swap::CurrencySwap;
use sp_std::marker::PhantomData;

pub struct OneToOne<AccountIdFrom, AccountIdTo, CurrencyFrom, CurrencyTo>(
    PhantomData<(AccountIdFrom, AccountIdTo, CurrencyFrom, CurrencyTo)>,
)
where
    CurrencyFrom: Currency<AccountIdFrom>,
    CurrencyTo: Currency<AccountIdTo>;

impl<AccountIdFrom, AccountIdTo, CurrencyFrom, CurrencyTo> CurrencySwap<AccountIdFrom, AccountIdTo>
    for OneToOne<AccountIdFrom, AccountIdTo, CurrencyFrom, CurrencyTo>
where
    CurrencyFrom: Currency<AccountIdFrom>,
    CurrencyTo: Currency<AccountIdTo>,
{
    type From = CurrencyFrom;
    type To = CurrencyTo;
    type Error = DispatchError;

    fn swap(
        imbalance: <Self::From as frame_support::traits::Currency<AccountIdFrom>>::NegativeImbalance,
    ) -> Result<
        <Self::To as frame_support::traits::Currency<AccountIdTo>>::NegativeImbalance,
        Self::Error,
    > {
        todo!()
    }
}
