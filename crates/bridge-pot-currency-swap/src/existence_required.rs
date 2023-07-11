//! An implementation that requires and ensures pot account existence.

use frame_support::{
    sp_runtime::{traits::Convert, DispatchError},
    traits::{fungible::Balanced, Get, Imbalance},
};

use super::{Config, CurrencySwap};

/// A marker type for the implementation that requires pot accounts existence.
pub enum Marker {}

/// An error that can occur while doing the swap operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Unable to resolve the incoming balance into the corresponding pot.
    ResolvingIncomingCredit,
    /// Unable to withdraw the outgoing balance from the corresponding pot.
    IssuingOutgoingCredit(DispatchError),
}

impl From<Error> for DispatchError {
    fn from(value: Error) -> Self {
        match value {
            Error::ResolvingIncomingCredit => {
                DispatchError::Other("swap pot account does not exist")
            }
            Error::IssuingOutgoingCredit(err) => err,
        }
    }
}

impl<T: Config> primitives_currency_swap::CurrencySwap<T::AccountIdFrom, T::AccountIdTo>
    for CurrencySwap<T, Marker>
{
    type From = T::CurrencyFrom;
    type To = T::CurrencyTo;
    type Error = Error;

    fn swap(
        incoming_credit: primitives_currency_swap::FromCreditOf<
            Self,
            T::AccountIdFrom,
            T::AccountIdTo,
        >,
    ) -> Result<
        primitives_currency_swap::ToCreditOf<Self, T::AccountIdFrom, T::AccountIdTo>,
        primitives_currency_swap::ErrorFor<Self, T::AccountIdFrom, T::AccountIdTo>,
    > {
        let amount = incoming_credit.peek();

        let outgoing_credit = match T::CurrencyTo::withdraw(
            &T::PotTo::get(),
            T::BalanceConverter::convert(amount),
            /* ExistenceRequirement::KeepAlive */
        ) {
            Ok(outgoing_credit) => outgoing_credit,
            Err(error) => {
                return Err(primitives_currency_swap::Error {
                    cause: Error::IssuingOutgoingCredit(error),
                    incoming_credit,
                });
            }
        };

        match T::CurrencyFrom::resolve(&T::PotFrom::get(), incoming_credit) {
            Ok(()) => {}
            Err(incoming_credit) => {
                if let Err(_outgoing_credit) =
                    T::CurrencyTo::resolve(&T::PotTo::get(), outgoing_credit)
                {
                    // We have just withdrawn these funds, so we must be able to resolve them back.
                    unreachable!();
                }
                return Err(primitives_currency_swap::Error {
                    cause: Error::ResolvingIncomingCredit,
                    incoming_credit,
                });
            }
        }

        Ok(outgoing_credit)
    }
}
