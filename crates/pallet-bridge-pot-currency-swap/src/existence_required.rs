//! An implementation that requires and ensures pot account existence.

use frame_support::{
    sp_runtime::{traits::Convert, DispatchError},
    traits::{Currency, ExistenceRequirement, Get, Imbalance, WithdrawReasons},
};

use super::{Config, CurrencySwap, Pallet};

/// A marker type for the implementation that requires pot accounts existence.
pub enum Marker {}

/// An error that can occur while doing the swap operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Unable to resolve the incoming balance into the corresponding pot.
    ResolvingIncomingImbalance,
    /// Unable to withdraw the outgoing balance from the corresponding pot.
    IssuingOutgoingImbalance(DispatchError),
}

impl From<Error> for DispatchError {
    fn from(value: Error) -> Self {
        match value {
            Error::ResolvingIncomingImbalance => {
                DispatchError::Other("swap pot account does not exist")
            }
            Error::IssuingOutgoingImbalance(err) => err,
        }
    }
}

impl<T: Config<I>, I: 'static>
    primitives_currency_swap::CurrencySwap<T::AccountIdFrom, T::AccountIdTo>
    for CurrencySwap<Pallet<T, I>, Marker>
{
    type From = T::CurrencyFrom;
    type To = T::CurrencyTo;
    type Error = Error;

    fn swap(
        incoming_imbalance: <Self::From as Currency<T::AccountIdFrom>>::NegativeImbalance,
    ) -> Result<
        <Self::To as Currency<T::AccountIdTo>>::NegativeImbalance,
        primitives_currency_swap::ErrorFor<Self, T::AccountIdFrom, T::AccountIdTo>,
    > {
        let amount = incoming_imbalance.peek();

        let outgoing_imbalance = match T::CurrencyTo::withdraw(
            &T::PotTo::get(),
            T::BalanceConverter::convert(amount),
            WithdrawReasons::TRANSFER,
            ExistenceRequirement::KeepAlive,
        ) {
            Ok(imbalance) => imbalance,
            Err(error) => {
                return Err(primitives_currency_swap::Error {
                    cause: Error::IssuingOutgoingImbalance(error),
                    incoming_imbalance,
                });
            }
        };

        match T::CurrencyFrom::resolve_into_existing(&T::PotFrom::get(), incoming_imbalance) {
            Ok(()) => {}
            Err(imbalance) => {
                T::CurrencyTo::resolve_creating(&T::PotTo::get(), outgoing_imbalance);
                return Err(primitives_currency_swap::Error {
                    cause: Error::ResolvingIncomingImbalance,
                    incoming_imbalance: imbalance,
                });
            }
        }

        Ok(outgoing_imbalance)
    }

    fn estimate_swapped_balance(
        balance: <Self::From as Currency<T::AccountIdFrom>>::Balance,
    ) -> <Self::To as Currency<T::AccountIdTo>>::Balance {
        T::BalanceConverter::convert(balance)
    }
}
