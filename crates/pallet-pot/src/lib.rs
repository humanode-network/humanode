//! A pot - an instanced pallet indended to provide an govern a "system" account where some balance
//! can be sent.
//!
//! Intended for use as an implementation for the treasury, fee pot, etc.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Imbalance, OnUnbalanced, StorageVersion};
use frame_support::{pallet_prelude::*, traits::Currency, PalletId};
pub use pallet::*;
use sp_runtime::traits::{AccountIdConversion, Saturating};

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

/// The balance of accessor for the currency.
pub type BalanceOf<T, I = ()> =
    <<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// The positive imbalance accessor.
pub type PositiveImbalanceOf<T, I = ()> = <<T as Config<I>>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::PositiveImbalance;

/// The negative implanace accessor.
pub type NegativeImbalanceOf<T, I = ()> = <<T as Config<I>>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency to operate with.
        type Currency: Currency<Self::AccountId>;

        /// The pot's pallet id, used for deriving its sovereign account ID.
        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T, I = ()>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// Some funds have been deposited.
        Deposit {
            /// The amonut of funds that has been deposited.
            value: BalanceOf<T, I>,
        },
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
        /// The initial amount of funds in the pot, must be greater or equal to the existential
        /// deposit.
        pub initial_balance: BalanceOf<T, I>,
    }

    #[cfg(feature = "std")]
    impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
        fn default() -> Self {
            Self {
                initial_balance: T::Currency::minimum_balance(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
        fn build(&self) {
            // Create the pot account.
            let account_id = <Pallet<T, I>>::account_id();
            let min = T::Currency::minimum_balance();
            assert!(
                self.initial_balance >= min,
                "the initial pot balance must be greater or equal than the existential balance"
            );
            let _ = T::Currency::make_free_balance_be(&account_id, self.initial_balance);
        }
    }
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    /// The account ID of the pot.
    ///
    /// This actually does computation. If you need to keep using it, then make sure you cache the
    /// value and only call this once.
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account_truncating()
    }

    /// Return the amount of money in the pot.
    // The existential deposit is not part of the pot so treasury account never gets deleted.
    pub fn pot() -> BalanceOf<T, I> {
        T::Currency::free_balance(&Self::account_id())
            // Must never be less than 0 but better be safe.
            .saturating_sub(T::Currency::minimum_balance())
    }
}

impl<T: Config<I>, I: 'static> OnUnbalanced<NegativeImbalanceOf<T, I>> for Pallet<T, I> {
    fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<T, I>) {
        let numeric_amount = amount.peek();

        // Must resolve into existing but better to be safe.
        T::Currency::resolve_creating(&Self::account_id(), amount);

        Self::deposit_event(Event::Deposit {
            value: numeric_amount,
        });
    }
}
