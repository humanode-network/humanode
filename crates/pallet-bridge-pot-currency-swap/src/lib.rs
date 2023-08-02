//! A substrate pallet for bridge pot currency swap implementation.

// Either generate code at stadard mode, or `no_std`, based on the `std` feature presence.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    sp_runtime::traits::Convert,
    sp_std::marker::PhantomData,
    traits::{fungible::Inspect, Currency, StorageVersion},
};
mod balances_values;
pub mod existence_optional;
pub mod existence_required;
mod upgrade_init;

pub use self::balances_values::{Balanced, Error as BalancedError};
pub use self::existence_optional::Marker as ExistenceOptional;
pub use self::existence_required::Marker as ExistenceRequired;
pub use self::pallet::*;
pub use self::upgrade_init::InitBalanceProvider;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, sp_runtime::traits::MaybeDisplay};
    use frame_system::pallet_prelude::*;
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

        /// The balance provider for the pot initialization.
        type InitBalanceProvider: InitBalanceProvider<Self::AccountIdTo, Self::CurrencyTo>;
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
            let pot_to_balance = T::CurrencyTo::total_balance(&T::PotTo::get());
            match Balanced::<T, I>::expected_bridge_balance_at_to() {
                Ok(expected_pot_to_balance) => assert!(
                    pot_to_balance == expected_pot_to_balance,
                    "genesis bridge pot balances values not balanced"
                ),
                Err(err) => panic!(
                    "error during bridge pot balance calculation: {}",
                    Into::<&'static str>::into(err)
                ),
            }
        }
    }

    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
        fn on_runtime_upgrade() -> Weight {
            upgrade_init::on_runtime_upgrade::<T, I>()
        }

        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
            upgrade_init::pre_upgrade::<T, I>()
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
            upgrade_init::post_upgrade::<T, I>()
        }
    }
}

/// A [`primitives_currency_swap::CurrencySwap`] implementation that does the swap using two
/// "pot" accounts for each of end swapped currencies.
pub struct CurrencySwap<Pallet, ExistenceRequirement>(PhantomData<(Pallet, ExistenceRequirement)>);
