//! A pot - an instanced pallet intended to provide an govern a "system" account where some balance
//! can be sent.
//!
//! Intended for use as an implementation for the treasury, fee pot, etc.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    pallet_prelude::*,
    traits::{
        fungible::{Balanced, Credit, Inspect},
        Currency, Imbalance, OnUnbalanced, StorageVersion,
    },
    PalletId,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{AccountIdConversion, CheckedSub, MaybeDisplay, Saturating};

pub use self::pallet::*;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

/// The balance of accessor for the currency.
pub type BalanceOf<T, I = ()> =
    <<T as Config<I>>::Currency as Currency<<T as Config<I>>::AccountId>>::Balance;

/// The negative implanace accessor.
pub type NegativeImbalanceOf<T, I = ()> =
    <<T as Config<I>>::Currency as Currency<<T as Config<I>>::AccountId>>::NegativeImbalance;

/// The credit accessor.
pub type CreditOf<T, I = ()> = Credit<<T as Config<I>>::AccountId, <T as Config<I>>::Currency>;

/// The initial state of the pot, for use in genesis.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InitialState<Balance> {
    /// The state of the pot account is not checked at genesis.
    Unchecked,
    /// The account will be guaranteed to exist at genesis.
    Initialized,
    /// The account will be guaranteed to be initialize with the given balance at genesis.
    Balance {
        /// The balance to set for the account.
        balance: Balance,
    },
}

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use sp_std::fmt::Debug;

    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self, I>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The user account identifier type.
        type AccountId: Parameter
            + Member
            + MaybeSerializeDeserialize
            + Debug
            + MaybeDisplay
            + Ord
            + MaxEncodedLen;

        /// The fungible asset to operate with.
        type Currency: Currency<<Self as Config<I>>::AccountId>
            + Balanced<<Self as Config<I>>::AccountId, Balance = BalanceOf<Self, I>>;

        /// The pot's pallet id, used for deriving its sovereign account ID.
        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    #[pallet::extra_constants]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// The account ID of the pot.
        ///
        /// This actually performs computation.
        /// If you need to keep using it, then make sure you cache the value and
        /// only call this once.
        pub fn account_id() -> <T as Config<I>>::AccountId {
            T::PalletId::get().into_account_truncating()
        }
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T, I = ()>(_);

    /// The amount which has been reported as inactive to `T::Currency`.
    #[pallet::storage]
    pub type Inactive<T: Config<I>, I: 'static = ()> = StorageValue<_, BalanceOf<T, I>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// Some funds have been deposited.
        Deposit {
            /// The amount of funds that has been deposited.
            value: BalanceOf<T, I>,
        },
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
        /// The initial state of the pot account.
        pub initial_state: InitialState<BalanceOf<T, I>>,
    }

    impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
        fn default() -> Self {
            Self {
                initial_state: InitialState::Initialized,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
        fn build(&self) {
            // Check the pot account.
            let account_id = <Pallet<T, I>>::account_id();

            match self.initial_state {
                InitialState::Unchecked => {
                    // Just pass though.
                }
                InitialState::Initialized => {
                    let current = T::Currency::free_balance(&account_id);
                    let min = <T::Currency as Currency<_>>::minimum_balance();
                    assert!(
                        current >= min,
                        "the initial pot balance must be greater or equal than the existential balance"
                    );
                }
                InitialState::Balance { balance } => {
                    let min = <T::Currency as Currency<_>>::minimum_balance();
                    assert!(
                        balance >= min,
                        "the configured initial pot balance must be greater or equal than the existential balance"
                    );
                    let _ = T::Currency::make_free_balance_be(&account_id, balance);
                }
            }
        }
    }

    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            Self::update_inactive()
        }
    }
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    /// Return the balance currently stored in the pot.
    // The existential deposit (`minimum_balance`) is not part of
    // the pot so the pot account never gets killed.
    pub fn balance() -> BalanceOf<T, I> {
        T::Currency::balance(&Self::account_id())
            // Must never be less than 0 but better be safe.
            .saturating_sub(<T::Currency as Currency<_>>::minimum_balance())
    }

    /// Update the inactive supply for this pot.
    ///
    /// This function uses the whole balance of the account, unlike [`Self::balance`],
    /// which subtracts the existential balance.
    fn update_inactive() -> Weight {
        let balance = T::Currency::free_balance(&Self::account_id());
        let current = Inactive::<T, I>::get();

        let mut weight = T::DbWeight::get().reads(2);

        if balance != current {
            if let Some(delta) = balance.checked_sub(&current) {
                <T::Currency as Currency<_>>::deactivate(delta)
            }
            if let Some(delta) = current.checked_sub(&balance) {
                <T::Currency as Currency<_>>::reactivate(delta)
            }
            Inactive::<T, I>::put(balance);

            weight.saturating_accrue(T::DbWeight::get().writes(2));
        }

        weight
    }
}

/// Handle unbalanced funds by depositing them into this pot.
///
/// Implementation for [`Currency`].
pub struct DepositUnbalancedCurrency<T, I>(PhantomData<(T, I)>);

impl<T: Config<I>, I: 'static> OnUnbalanced<NegativeImbalanceOf<T, I>>
    for DepositUnbalancedCurrency<T, I>
{
    fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<T, I>) {
        let numeric_amount = amount.peek();

        // Must resolve into existing but better to be safe.
        T::Currency::resolve_creating(&Pallet::<T, I>::account_id(), amount);

        Pallet::<T, I>::deposit_event(Event::Deposit {
            value: numeric_amount,
        });
    }
}

/// Handle unbalanced funds by depositing them into this pot.
///
/// Implementation for `Fungible` behavior.
pub struct DepositUnbalancedFungible<T, I>(PhantomData<(T, I)>);

impl<T: Config<I>, I: 'static> OnUnbalanced<CreditOf<T, I>> for DepositUnbalancedFungible<T, I> {
    fn on_nonzero_unbalanced(amount: CreditOf<T, I>) {
        let numeric_amount = amount.peek();

        // Pot account already exists.
        let _ = T::Currency::resolve(&Pallet::<T, I>::account_id(), amount);

        Pallet::<T, I>::deposit_event(Event::Deposit {
            value: numeric_amount,
        });
    }
}
