//! Bridge pot balances values logic implementation.

use frame_support::{
    sp_runtime::{
        traits::{CheckedAdd, CheckedSub, Convert, Zero},
        ArithmeticError,
    },
    sp_std::marker::PhantomData,
    traits::{Currency, Get},
};

use super::Config;

/// The implementation that requires bridge pot balances values be balanced.
pub struct Balanced<T: Config<I>, I: 'static>(PhantomData<(T, I)>);

/// The bridge pot balances logic related error kinds.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// The arithmetic operation error.
    Arithmetic(ArithmeticError),
}

impl sp_std::fmt::Display for Error {
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        match self {
            Error::Arithmetic(err) => {
                write!(
                    f,
                    "An arithmetic error has been occured: {}",
                    Into::<&'static str>::into(*err)
                )
            }
        }
    }
}

impl<T: Config<I>, I: 'static> Balanced<T, I> {
    /// A function to calculate expected [`Config::PotTo`] balance based on the provided.
    pub fn expected_bridge_balance(
    ) -> Result<<T::CurrencyTo as Currency<T::AccountIdTo>>::Balance, Error> {
        let total_from = T::CurrencyFrom::total_issuance();
        let pot_from = T::CurrencyFrom::total_balance(&T::PotFrom::get());

        let ed_to = T::CurrencyTo::minimum_balance();

        let bridge_balance = T::BalanceConverter::convert(
            total_from
                .checked_sub(&pot_from)
                .ok_or(Error::Arithmetic(ArithmeticError::Underflow))?,
        )
        .checked_add(&ed_to)
        .ok_or(Error::Arithmetic(ArithmeticError::Overflow))?;

        Ok(bridge_balance)
    }

    /// A function to calculate balanced value based on the provided list of [`Config::AccountIdFrom`] balances.
    pub fn balanced_value(
        from_balances: impl IntoIterator<
            Item = <T::CurrencyFrom as Currency<T::AccountIdFrom>>::Balance,
        >,
    ) -> Result<<T::CurrencyTo as Currency<T::AccountIdTo>>::Balance, Error> {
        let to_bridge_balance = from_balances.into_iter().try_fold(
            Zero::zero(),
            |sum: <T::CurrencyFrom as Currency<T::AccountIdFrom>>::Balance, from_balance| {
                sum.checked_add(&from_balance)
                    .ok_or(Error::Arithmetic(ArithmeticError::Overflow))
            },
        )?;

        let to_bridge_balance = T::BalanceConverter::convert(to_bridge_balance)
            .checked_add(&T::CurrencyTo::minimum_balance())
            .ok_or(Error::Arithmetic(ArithmeticError::Overflow))?;

        Ok(to_bridge_balance)
    }
}
