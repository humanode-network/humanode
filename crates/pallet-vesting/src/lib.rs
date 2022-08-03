//! Vesting.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, LockIdentifier, LockableCurrency, StorageVersion};

pub use self::pallet::*;

pub mod traits;
pub mod types;
pub mod weights;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

/// The currency from a given config.
type CurrencyOf<T> = <T as Config>::Currency;
/// The Account ID from a given config.
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
/// The balance from a given config.
type BalanceOf<T> = <CurrencyOf<T> as Currency<AccountIdOf<T>>>::Balance;
/// The lock info from a given config.
type LockInfoOf<T> = types::LockInfo<BalanceOf<T>, <T as Config>::Schedule>;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*, sp_runtime::traits::Zero, storage::transactional::in_storage_layer,
        traits::WithdrawReasons,
    };
    use frame_system::pallet_prelude::*;

    use super::*;
    use crate::{traits::SchedulingDriver, weights::WeightInfo};

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Currency to claim.
        type Currency: LockableCurrency<<Self as frame_system::Config>::AccountId>;

        /// The ID of the lock to use at the lockable balance.
        type LockId: Get<LockIdentifier>;

        /// The vesting schedule configration type.
        type Schedule: Member + Parameter + MaxEncodedLen + MaybeSerializeDeserialize;

        /// The scheduling driver to use for computing balance unlocks.
        type SchedulingDriver: SchedulingDriver<
            Balance = BalanceOf<Self>,
            Schedule = Self::Schedule,
        >;

        /// The weight informtation provider type.
        type WeightInfo: WeightInfo;
    }

    /// The locks information.
    #[pallet::storage]
    #[pallet::getter(fn locks)]
    pub type Locks<T> = StorageMap<_, Twox64Concat, AccountIdOf<T>, LockInfoOf<T>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Balance was locked under vesting.
        Locked {
            /// Who had the balance unlocked.
            who: T::AccountId,
            /// The balance that was unlocked.
            balance: BalanceOf<T>,
            /// The unlocking schedule.
            schedule: T::Schedule,
        },
        /// Vested balance was partially unlocked.
        PartiallyUnlocked {
            /// Who had the balance unlocked.
            who: T::AccountId,
            /// The balance that still remains locked.
            balance_left_under_lock: BalanceOf<T>,
        },
        /// Vesting is over and the locked balance has been fully unlocked.
        FullyUnlocked {
            /// Who had the vesting.
            who: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Vesting is already engaged for a given account.
        VestingAlreadyEngaged,

        /// Locking zero balance under vesting in prohibited.
        LockingZeroBalance,

        /// No vesting is active for a given account.
        NoVesting,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Unlock the vested balance according to the schedule.
        #[pallet::weight(T::WeightInfo::unlock())]
        pub fn unlock(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::unlock_vested_balance(&who)
        }
    }

    impl<T: Config> Pallet<T> {
        /// Lock the specified balance at the given account under vesting.
        ///
        /// This will fully lock the balance in question, allowing the user to invoke unlocking
        /// as needed.
        ///
        /// Only one vesting lock per account can exist at a time.
        /// Locking zero balance is prohibited in this implementation.
        pub fn lock_under_vesting(
            who: &T::AccountId,
            amount: BalanceOf<T>,
            schedule: T::Schedule,
        ) -> DispatchResult {
            in_storage_layer(|| {
                // Check if we're locking zero balance.
                if amount == Zero::zero() {
                    return Err(<Error<T>>::LockingZeroBalance.into());
                }

                // Check if a given account already has vesting engaged.
                if <Locks<T>>::contains_key(who) {
                    return Err(<Error<T>>::VestingAlreadyEngaged.into());
                }

                // Store the schedule.
                <Locks<T>>::insert(
                    who,
                    types::LockInfo {
                        initial_locked_balance: amount,
                        schedule,
                    },
                );

                // Fully lock the balance initially, disregarding the unlock schedule at this time.
                Self::set_lock(who, amount);

                Ok(())
            })
        }

        /// Unlock the vested balance on a given account according to the unlocking schedule.
        ///
        /// If the balance left under lock is zero, the lock is removed along with the vesting
        /// information - effectively eliminating any effect this pallet has on the given account's
        /// balance.
        ///
        /// If the balance left under lock is non-zero we keep the readjust the lock and keep
        /// the vesting information around.
        pub fn unlock_vested_balance(who: &T::AccountId) -> DispatchResult {
            in_storage_layer(|| {
                let lock_info = <Locks<T>>::get(who).ok_or(<Error<T>>::NoVesting)?;

                // Compute the new locked balance.
                let computed_locked_balance = T::SchedulingDriver::compute_balance_under_lock(
                    lock_info.initial_locked_balance,
                    &lock_info.schedule,
                )?;

                // If we ended up locking the whole balance we are done with the vesting.
                // Clean up the state and unlock the whole balance.
                if computed_locked_balance == Zero::zero() {
                    // Remove the lock info.
                    <Locks<T>>::remove(who);

                    // Remove the balance lock.
                    <CurrencyOf<T> as LockableCurrency<T::AccountId>>::remove_lock(
                        T::LockId::get(),
                        who,
                    );

                    // Dispatch the event.
                    Self::deposit_event(Event::FullyUnlocked { who: who.clone() });

                    // We're done!
                    return Ok(());
                }

                // Set the lock to the new value if the balance to lock is non-zero.
                Self::set_lock(who, computed_locked_balance);

                // Dispatch the event.
                Self::deposit_event(Event::PartiallyUnlocked {
                    who: who.clone(),
                    balance_left_under_lock: computed_locked_balance,
                });

                Ok(())
            })
        }

        fn set_lock(who: &T::AccountId, balance_to_lock: BalanceOf<T>) {
            debug_assert!(
                balance_to_lock != Zero::zero(),
                "we must ensure that the balance is non-zero when calling this fn"
            );

            // Set the lock.
            <CurrencyOf<T> as LockableCurrency<T::AccountId>>::set_lock(
                T::LockId::get(),
                who,
                balance_to_lock,
                WithdrawReasons::all(),
            );
        }
    }
}
