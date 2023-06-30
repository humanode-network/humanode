//! An implementation that requires and ensures pot account existence.

use frame_support::{
    sp_runtime::{traits::Convert, DispatchError},
    traits::{Currency, ExistenceRequirement, Get, Imbalance, WithdrawReasons},
};

use super::{Config, CurrencySwap};

/// A marker type for the implementation that requires pot accounts existence.
pub enum Marker {}

/// An error that can occur while doing the swap operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error<ImbalanceFrom> {
    /// Unable to resolve the incoming balance into the corresponding pot.
    ResolvingIncomingImbalance(ImbalanceFrom),
    /// Unable to withdraw the outpoing balance from the corresponding pot.
    IssuingOutgoingImbalance(DispatchError),
}

impl<T> From<Error<T>> for DispatchError {
    fn from(value: Error<T>) -> Self {
        match value {
            Error::ResolvingIncomingImbalance(_) => DispatchError::NoProviders,
            Error::IssuingOutgoingImbalance(err) => err,
        }
    }
}

impl<T: Config> primitives_currency_swap::CurrencySwap<T::AccountIdFrom, T::AccountIdTo>
    for CurrencySwap<T, Marker>
{
    type From = T::CurrencyFrom;
    type To = T::CurrencyTo;
    type Error =
        Error<<<T as Config>::CurrencyFrom as Currency<T::AccountIdFrom>>::NegativeImbalance>;

    fn swap(
        imbalance: <Self::From as Currency<T::AccountIdFrom>>::NegativeImbalance,
    ) -> Result<<Self::To as Currency<T::AccountIdTo>>::NegativeImbalance, Self::Error> {
        let amount = imbalance.peek();

        T::CurrencyFrom::resolve_into_existing(&T::PotFrom::get(), imbalance)
            .map_err(Error::ResolvingIncomingImbalance)?;

        let imbalance = T::CurrencyTo::withdraw(
            &T::PotTo::get(),
            T::BalanceCoverter::convert(amount),
            WithdrawReasons::TRANSFER,
            ExistenceRequirement::KeepAlive,
        )
        .map_err(Error::IssuingOutgoingImbalance)?;

        Ok(imbalance)
    }
}
