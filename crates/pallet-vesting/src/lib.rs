//! Vesting.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, LockIdentifier, LockableCurrency, StorageVersion};
pub use weights::*;

pub use self::logic::*;
pub use self::pallet::*;

pub mod api;
mod logic;
pub mod traits;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
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
    use frame_support::{pallet_prelude::*, storage::transactional::in_storage_layer};
    use frame_system::pallet_prelude::*;

    use super::*;
    use crate::{traits::SchedulingDriver, weights::WeightInfo};

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Currency to claim.
        type Currency: LockableCurrency<<Self as frame_system::Config>::AccountId>;

        /// The ID of the lock to use at the lockable balance.
        type LockId: Get<LockIdentifier>;

        /// The vesting schedule configuration type.
        type Schedule: Member + Parameter + MaxEncodedLen + MaybeSerializeDeserialize;

        /// The scheduling driver to use for computing balance unlocks.
        type SchedulingDriver: SchedulingDriver<
            Balance = BalanceOf<Self>,
            Schedule = Self::Schedule,
        >;

        /// The weight information provider type.
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
        /// Vesting schedule has been updated.
        VestingUpdated {
            /// Account with locked balance that got its schedule updated.
            account_id: T::AccountId,
            /// The old vesting schedule.
            old_schedule: T::Schedule,
            /// The new vesting schedule.
            new_schedule: T::Schedule,
            /// The balance that is locked under vesting with new schedule.
            balance_under_lock: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Vesting is already engaged for a given account.
        VestingAlreadyEngaged,

        /// No vesting is active for a given account.
        NoVesting,
    }

    #[pallet::call(weight(T::WeightInfo))]
    impl<T: Config> Pallet<T> {
        /// Unlock the vested balance according to the schedule.
        #[pallet::call_index(0)]
        pub fn unlock(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let schedule = <Schedules<T>>::get(&who).ok_or(<Error<T>>::NoVesting)?;

            in_storage_layer(|| {
                let effect = Self::compute_effect(&schedule)?;
                Self::apply_effect(Operation::Unlock(effect, &who));
                Ok(())
            })
        }

        /// Update existing vesting with the provided schedule.
        ///
        /// Root-level operation.
        #[pallet::call_index(1)]
        pub fn update_schedule(
            origin: OriginFor<T>,
            account_id: T::AccountId,
            new_schedule: T::Schedule,
        ) -> DispatchResult {
            ensure_root(origin)?;

            in_storage_layer(|| {
                let old_schedule = <Schedules<T>>::get(&account_id).ok_or(<Error<T>>::NoVesting)?;

                let effect = Self::compute_effect(&new_schedule)?;
                let balance_under_lock = effect.effective_balance_under_lock();
                Self::apply_effect(Operation::Update(effect, new_schedule.clone(), &account_id));

                // Send the event announcing the update.
                Self::deposit_event(Event::VestingUpdated {
                    account_id,
                    old_schedule,
                    new_schedule,
                    balance_under_lock,
                });

                Ok(())
            })
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

                let effect = Self::compute_effect(&schedule)?;

                // Send the event announcing the lock.
                Self::deposit_event(Event::Locked {
                    who: who.clone(),
                    schedule: schedule.clone(),
                    // Note that we emit this event even if the locked balance is zero.
                    // The rationale for this is we indicate that the lock invocation was successful
                    // yet no balance was locked.
                    balance_under_lock: effect.effective_balance_under_lock(),
                });

                Self::apply_effect(Operation::Init(effect, schedule, who));

                Ok(())
            })
        }

        /// Evaluate the vesting logic and compute the locked balance.
        /// Intended for implementing the [`api::VestingEvaluationApi`].
        pub fn evaluate_lock(who: &T::AccountId) -> Result<BalanceOf<T>, api::EvaluationError> {
            let schedule = <Schedules<T>>::get(who).ok_or(api::EvaluationError::NoVesting)?;

            // Compute the new locked balance.
            let computed_locked_balance =
                T::SchedulingDriver::compute_balance_under_lock(&schedule)
                    .map_err(api::EvaluationError::Computation)?;

            Ok(computed_locked_balance)
        }
    }
}
