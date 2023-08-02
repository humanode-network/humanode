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
    /// The balance that at this moment can theoritically be sent to the [`PotFrom`].
    pub fn swappable_balance_at_from(
    ) -> Result<<T::CurrencyFrom as Currency<T::AccountIdFrom>>::Balance, ArithmeticError> {
        let total_from = T::CurrencyFrom::total_issuance();
        let bridge_from = T::CurrencyFrom::total_balance(&T::PotFrom::get());

        let swappable_at_from = total_from
            .checked_sub(&bridge_from)
            .ok_or(ArithmeticError::Underflow)?;

        Ok(swappable_at_from)
    }

    /// The balance that at this moment can theoritically be withdrawn from [`PotTo`].
    pub fn swappable_balance_at_to(
    ) -> Result<<T::CurrencyTo as Currency<T::AccountIdTo>>::Balance, ArithmeticError> {
        let bridge_to = T::CurrencyTo::total_balance(&T::PotTo::get());
        let ed_to = T::CurrencyTo::minimum_balance();

        let swappable_at_to = bridge_to
            .checked_sub(&ed_to)
            .ok_or(ArithmeticError::Underflow)?;

        Ok(swappable_at_to)
    }

    /// The expected bridge balance.
    pub fn expected_bridge_balance_at_to(
    ) -> Result<<T::CurrencyTo as Currency<T::AccountIdTo>>::Balance, ArithmeticError> {
        let to_ed = T::CurrencyTo::minimum_balance();
        let swappable_at_from = T::BalanceConverter::convert(Self::swappable_balance_at_from()?);
        to_ed
            .checked_add(&swappable_at_from)
            .ok_or(ArithmeticError::Underflow)
    }

    /// Ensure the swappable from and to and in balance.
    pub fn verify_swappable_balance() -> Result<bool, ArithmeticError> {
        let swappable_at_from = T::BalanceConverter::convert(Self::swappable_balance_at_from()?);
        let swappable_at_to = Self::swappable_balance_at_to()?;
        let is_balanced = swappable_at_from == swappable_at_to;
        Ok(is_balanced)
    }

    /// A function to calculate balanced value based on the provided list of [`Config::AccountIdFrom`] balances.
    pub fn genesis_swappable_balance_at_from(
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
