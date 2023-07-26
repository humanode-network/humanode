//! An implementation that verifies bridge pot balance values at genesis.

use frame_support::{
    sp_runtime::{
        traits::{CheckedAdd, Convert, Zero},
        ArithmeticError, DispatchError,
    },
    sp_std::marker::PhantomData,
    traits::{Currency, Get},
};

use super::{Config, GenesisVerifier, Pallet};

/// The implementation that requires bridge pot balance values be balanced.
pub struct Balanced<Pallet>(PhantomData<Pallet>);

impl<T: Config<I>, I: 'static> Balanced<Pallet<T, I>> {
    /// A function to calculate expected [`Config::PotTo`] balance based on the provided list of
    /// all [`Config::AccountIdFrom`] balances except [`Config::PotFrom`] balance.
    pub fn calculate_expected_to_bridge_balance(
        all_from_balances_without_bridge_balance: &[<T::CurrencyFrom as Currency<
            T::AccountIdFrom,
        >>::Balance],
    ) -> Result<<T::CurrencyTo as Currency<T::AccountIdTo>>::Balance, DispatchError> {
        let to_bridge_balance = all_from_balances_without_bridge_balance.iter().try_fold(
            Zero::zero(),
            |sum: <T::CurrencyFrom as Currency<T::AccountIdFrom>>::Balance, &from_balance| {
                sum.checked_add(&from_balance)
                    .ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))
            },
        )?;

        let to_bridge_balance = T::BalanceConverter::convert(to_bridge_balance)
            .checked_add(&T::CurrencyTo::minimum_balance())
            .ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?;

        Ok(to_bridge_balance)
    }
}

impl<T: Config<I>, I: 'static> GenesisVerifier for Balanced<Pallet<T, I>> {
    fn verify() -> bool {
        let total_from = T::CurrencyFrom::total_issuance();
        let total_to = T::CurrencyTo::total_issuance();

        let pot_from = T::CurrencyFrom::total_balance(&T::PotFrom::get());
        let pot_to = T::CurrencyTo::total_balance(&T::PotTo::get());

        let ed_from = T::CurrencyFrom::minimum_balance();
        let ed_to = T::CurrencyTo::minimum_balance();

        (T::BalanceConverter::convert(pot_from - ed_from) == (total_to - pot_to))
            && ((pot_to - ed_to) == T::BalanceConverter::convert(total_from - pot_from))
    }
}
