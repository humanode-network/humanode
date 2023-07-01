//! An implementation that does not require pot account existence and can potentially kill the
//! pot account by withdrawing all the funds from it.

use frame_support::{
    sp_runtime::{traits::Convert, DispatchError},
    traits::{Currency, ExistenceRequirement, Get, Imbalance, WithdrawReasons},
};

use super::{Config, CurrencySwap};

/// A marker type for the implementation that does not require pot accounts existence.
pub enum Marker {}

impl<T: Config> primitives_currency_swap::CurrencySwap<T::AccountIdFrom, T::AccountIdTo>
    for CurrencySwap<T, Marker>
{
    type From = T::CurrencyFrom;
    type To = T::CurrencyTo;
    type Error = DispatchError;

    fn swap(
        imbalance: <Self::From as Currency<T::AccountIdFrom>>::NegativeImbalance,
    ) -> Result<<Self::To as Currency<T::AccountIdTo>>::NegativeImbalance, Self::Error> {
        let amount = imbalance.peek();

        T::CurrencyFrom::resolve_creating(&T::PotFrom::get(), imbalance);

        let imbalance = T::CurrencyTo::withdraw(
            &T::PotTo::get(),
            T::BalanceCoverter::convert(amount),
            WithdrawReasons::TRANSFER,
            ExistenceRequirement::AllowDeath,
        )?;

        Ok(imbalance)
    }
}
