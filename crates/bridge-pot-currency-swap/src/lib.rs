//! Bridge pot currency swap implementation.

// Either generate code at stadard mode, or `no_std`, based on the `std` feature presence.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    sp_runtime::DispatchError,
    traits::{Currency, ExistenceRequirement, Get, Imbalance, WithdrawReasons},
};
use primitives_currency_swap::CurrencySwap;
use sp_std::marker::PhantomData;

pub trait Config<AccountIdFrom, AccountIdTo> {
    type PotFrom: Get<AccountIdFrom>;

    type PotTo: Get<AccountIdTo>;
}

pub struct OneToOne<AccountIdFrom, AccountIdTo, ConfigT, CurrencyFrom, CurrencyTo>(
    PhantomData<(
        AccountIdFrom,
        AccountIdTo,
        ConfigT,
        CurrencyFrom,
        CurrencyTo,
    )>,
)
where
    ConfigT: Config<AccountIdFrom, AccountIdTo>,
    CurrencyFrom: Currency<AccountIdFrom>,
    CurrencyTo: Currency<AccountIdTo>;

impl<AccountIdFrom, AccountIdTo, ConfigT, CurrencyFrom, CurrencyTo>
    CurrencySwap<AccountIdFrom, AccountIdTo>
    for OneToOne<AccountIdFrom, AccountIdTo, ConfigT, CurrencyFrom, CurrencyTo>
where
    ConfigT: Config<AccountIdFrom, AccountIdTo>,
    CurrencyFrom: Currency<AccountIdFrom>,
    CurrencyTo: Currency<AccountIdTo>,
    CurrencyTo::Balance: From<CurrencyFrom::Balance>,
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
        let amount = imbalance.peek();

        CurrencyFrom::resolve_creating(&ConfigT::PotFrom::get(), imbalance);

        let imbalance = CurrencyTo::withdraw(
            &ConfigT::PotTo::get(),
            amount.into(),
            WithdrawReasons::TRANSFER,
            ExistenceRequirement::AllowDeath,
        )?;

        Ok(imbalance)
    }
}
