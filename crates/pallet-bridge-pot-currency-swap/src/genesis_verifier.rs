//! An implementation that verifies bridge pot balance values at genesis.

use frame_support::{
    sp_runtime::traits::Convert,
    sp_std::marker::PhantomData,
    traits::{Currency, Get},
};

use super::{Config, GenesisVerifier, Pallet};

/// The implementation that requires bridge pot balance values be balanced.
pub struct Balanced<Pallet>(PhantomData<Pallet>);

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
