//! A substrate pallet for bridge pot currency swap implementation.

// Either generate code at stadard mode, or `no_std`, based on the `std` feature presence.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    sp_runtime::{
        traits::{CheckedAdd, CheckedSub, Convert, Zero},
        ArithmeticError, DispatchError,
    },
    sp_std::marker::PhantomData,
    traits::{fungible::Inspect, Currency, Get, StorageVersion},
};
pub mod existence_optional;
pub mod existence_required;

pub use existence_optional::Marker as ExistenceOptional;
pub use existence_required::Marker as ExistenceRequired;
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, sp_runtime::traits::MaybeDisplay};
    use sp_std::fmt::Debug;

    use super::*;

    /// The Bridge Pot Currency Swap Pallet.
    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T, I = ()>(_);

    /// Configuration trait of this pallet.
    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        /// The type representing the account key for the currency to swap from.
        type AccountIdFrom: Parameter
            + Member
            + MaybeSerializeDeserialize
            + Debug
            + MaybeDisplay
            + Ord
            + MaxEncodedLen;

        /// The type representing the account key for the currency to swap to.
        type AccountIdTo: Parameter
            + Member
            + MaybeSerializeDeserialize
            + Debug
            + MaybeDisplay
            + Ord
            + MaxEncodedLen;

        /// The currency to swap from.
        type CurrencyFrom: Currency<Self::AccountIdFrom>
            + Inspect<
                Self::AccountIdFrom,
                Balance = <Self::CurrencyFrom as Currency<Self::AccountIdFrom>>::Balance,
            >;

        /// The currency to swap to.
        type CurrencyTo: Currency<Self::AccountIdTo>
            + Inspect<
                Self::AccountIdTo,
                Balance = <Self::CurrencyTo as Currency<Self::AccountIdTo>>::Balance,
            >;

        /// The converter to determine how the balance amount should be converted from one currency to
        /// another.
        type BalanceConverter: Convert<
            <Self::CurrencyFrom as Currency<Self::AccountIdFrom>>::Balance,
            <Self::CurrencyTo as Currency<Self::AccountIdTo>>::Balance,
        >;

        /// The account to land the balances to when receiving the funds as part of the swap operation.
        type PotFrom: Get<Self::AccountIdFrom>;

        /// The account to take the balances from when sending the funds as part of the swap operation.
        type PotTo: Get<Self::AccountIdTo>;
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config<I>, I: 'static = ()>(PhantomData<(T, I)>);

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }

    // The build of genesis for the pallet.
    #[pallet::genesis_build]
    impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
        fn build(&self) {
            let bridge_to_balance = T::CurrencyTo::total_balance(&T::PotTo::get());
            match Pallet::<T, I>::expected_bridge_to_balance() {
                Ok(expected_bridge_to_balance) => assert!(
                    bridge_to_balance == expected_bridge_to_balance,
                    "invalid bridge balance value: got {bridge_to_balance:?}, expected {expected_bridge_to_balance:?}"
                ),
                Err(err) => panic!(
                    "error during bridge balance calculation: {err:?}"
                ),
            }
        }
    }
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    /// A function to calculate expected [`Config::PotTo`] bridge balance.
    pub fn expected_bridge_to_balance(
    ) -> Result<<T::CurrencyTo as Currency<T::AccountIdTo>>::Balance, DispatchError> {
        let total_from = <T::CurrencyFrom as Currency<T::AccountIdFrom>>::total_issuance();
        let bridge_from = T::CurrencyFrom::total_balance(&T::PotFrom::get());

        let swappable_from = total_from
            .checked_sub(&bridge_from)
            .ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))?;

        let ed_to = <T::CurrencyTo as Currency<T::AccountIdTo>>::minimum_balance();

        let bridge_balance = T::BalanceConverter::convert(swappable_from)
            .checked_add(&ed_to)
            .ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?;

        Ok(bridge_balance)
    }

    /// A function to calculate genesis [`Config::PotTo`] bridge balance value based on the provided
    /// list of [`Config::AccountIdFrom`] genesis balances.
    pub fn genesis_bridge_to_balance(
        genesis_from_balances: impl IntoIterator<
            Item = <T::CurrencyFrom as Currency<T::AccountIdFrom>>::Balance,
        >,
    ) -> Result<<T::CurrencyTo as Currency<T::AccountIdTo>>::Balance, DispatchError> {
        let bridge_balance = genesis_from_balances.into_iter().try_fold(
            Zero::zero(),
            |sum: <T::CurrencyFrom as Currency<T::AccountIdFrom>>::Balance, from_balance| {
                sum.checked_add(&from_balance)
                    .ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))
            },
        )?;

        let bridge_balance = T::BalanceConverter::convert(bridge_balance)
            .checked_add(&<T::CurrencyTo as Currency<T::AccountIdTo>>::minimum_balance())
            .ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?;

        Ok(bridge_balance)
    }
}

/// A [`primitives_currency_swap::CurrencySwap`] implementation that does the swap using two
/// "pot" accounts for each of end swapped currencies.
pub struct CurrencySwap<Pallet, ExistenceRequirement>(PhantomData<(Pallet, ExistenceRequirement)>);
