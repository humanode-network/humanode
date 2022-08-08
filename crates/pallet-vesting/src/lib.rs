//! Vesting.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, LockIdentifier, LockableCurrency, StorageVersion};

pub use self::pallet::*;

pub mod traits;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

/// The currency from a given config.
type CurrencyOf<T> = <T as Config>::Currency;
/// The Account ID from a given config.
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
/// The balance from a given config.
type BalanceOf<T> = <CurrencyOf<T> as Currency<AccountIdOf<T>>>::Balance;

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

    /// The schedules information.
    #[pallet::storage]
    #[pallet::getter(fn locks)]
    pub type Schedules<T> =
        StorageMap<_, Twox64Concat, AccountIdOf<T>, <T as Config>::Schedule, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Balance was locked under vesting.
        Locked {
            /// Who had the balance locked.
            who: T::AccountId,
            /// The unlocking schedule.
            schedule: T::Schedule,
            /// The balance that is locked under vesting.
            balance_under_lock: BalanceOf<T>,
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

        /// Locking zero balance under vesting is prohibited.
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
        /// Lock the balance at the given account under the specified vesting schedule.
        ///
        /// The amount to lock depends on the actual schedule and will be computed on the fly.
        ///
        /// Only one vesting balance lock per account can exist at a time.
        ///
        /// Locking zero balance will skip creating the lock and will directly emit
        /// the "fully unlocked" event.
        pub fn lock_under_vesting(who: &T::AccountId, schedule: T::Schedule) -> DispatchResult {
            in_storage_layer(|| {
                // Check if a given account already has vesting engaged.
                if <Schedules<T>>::contains_key(who) {
                    return Err(<Error<T>>::VestingAlreadyEngaged.into());
                }

                // Compute the locked balance.
                let computed_locked_balance =
                    T::SchedulingDriver::compute_balance_under_lock(&schedule)?;

                // Send the event announcing the lock.
                Self::deposit_event(Event::Locked {
                    who: who.clone(),
                    schedule: schedule.clone(),
                    balance_under_lock: computed_locked_balance,
                });

                // Check if we're locking zero balance.
                if computed_locked_balance == Zero::zero() {
                    // If we do - skip creating the schedule and locking altogether.

                    // Send the unlock event.
                    Self::deposit_event(Event::FullyUnlocked { who: who.clone() });

                    return Ok(());
                }

                // Store the schedule.
                <Schedules<T>>::insert(who, schedule);

                // Set the lock.
                Self::set_lock(who, computed_locked_balance);

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
                let schedule = <Schedules<T>>::get(who).ok_or(<Error<T>>::NoVesting)?;

                // Compute the new locked balance.
                let computed_locked_balance =
                    T::SchedulingDriver::compute_balance_under_lock(&schedule)?;

                // If we ended up locking the whole balance we are done with the vesting.
                // Clean up the state and unlock the whole balance.
                if computed_locked_balance == Zero::zero() {
                    // Remove the schedule.
                    <Schedules<T>>::remove(who);

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

                // Set the lock to the updated value.
                Self::set_lock(who, computed_locked_balance);

                // Dispatch the event.
                Self::deposit_event(Event::PartiallyUnlocked {
                    who: who.clone(),
                    balance_left_under_lock: computed_locked_balance,
                });

                Ok(())
            })
        }

        pub(crate) fn set_lock(who: &T::AccountId, balance_to_lock: BalanceOf<T>) {
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
