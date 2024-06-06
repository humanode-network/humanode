//! Logic of the vesting (re)computation and effects.

use frame_support::{pallet_prelude::*, sp_runtime::traits::Zero, traits::WithdrawReasons};

use super::*;
use crate::traits::SchedulingDriver;

/// An operation on a vesting schedule that's undergo.
pub enum Operation<'a, T: Config> {
    /// Vesting is being initialized.
    Init(
        /// Effect of the initialization.
        Effect<BalanceOf<T>>,
        /// A schedule the initialization is conducted with.
        T::Schedule,
        /// An account the vesting it applied to.
        &'a T::AccountId,
    ),
    /// Vesting schedule is being updated.
    Update(
        /// Effect of the update.
        Effect<BalanceOf<T>>,
        /// An updated schedule the update operation is conducted with.
        T::Schedule,
        /// An account the vesting it updated for.
        &'a T::AccountId,
    ),
    /// Balance is being unlocked.
    Unlock(
        /// Effect of the unlock.
        Effect<BalanceOf<T>>,
        /// An account the unlock is performed for.
        &'a T::AccountId,
    ),
}

/// The effect of the schedule computation.
pub enum Effect<Balance> {
    /// The effect to apply after the computation is to execute a partial unlock of the balance.
    /// This implies that the vesting schedule will exist after the operation.
    PartialUnlock(Balance),
    /// The effect to apply after the computation is to execute a full unlock of the balance.
    /// This implies that the vesting schedule will not exist after the operation.
    FullUnlock,
}

impl<T: Zero + Copy> Effect<T> {
    /// Take a newly computed locked balance and derive an effective effect to apply.
    pub fn from_new_balance_under_lock(computed_locked_balance: T) -> Self {
        if computed_locked_balance.is_zero() {
            return Self::FullUnlock;
        }
        Self::PartialUnlock(computed_locked_balance)
    }

    /// Compute the effective lock amount based on the effect.
    pub fn effective_balance_under_lock(&self) -> T {
        match self {
            Self::FullUnlock => Zero::zero(),
            Self::PartialUnlock(val) => *val,
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Compute the effect to apply based on the schedule.
    pub(super) fn compute_effect(
        schedule: &T::Schedule,
    ) -> Result<Effect<BalanceOf<T>>, DispatchError> {
        // Compute the new locked balance.
        let computed_locked_balance = T::SchedulingDriver::compute_balance_under_lock(schedule)?;
        // Convert it to an effect.
        Ok(Effect::from_new_balance_under_lock(computed_locked_balance))
    }

    /// Apply the effect of a schedule in the context of an operation.
    ///
    /// This function encapsulates all the logic of applying the effects of the locked balance
    /// computation.
    pub(super) fn apply_effect(op: Operation<'_, T>) {
        match op {
            Operation::Init(Effect::PartialUnlock(balance_left_under_lock), schedule, who)
            | Operation::Update(Effect::PartialUnlock(balance_left_under_lock), schedule, who) => {
                // Store the initial or updated schedule.
                <Schedules<T>>::insert(who, schedule);

                // Set the lock and emit an update event.
                Self::execute_partial_unlock(who, balance_left_under_lock);
            }
            Operation::Unlock(Effect::PartialUnlock(balance_left_under_lock), who) => {
                // Set the lock and emit an update event.
                Self::execute_partial_unlock(who, balance_left_under_lock)
            }
            Operation::Init(Effect::FullUnlock, _, who) => {
                // Remove the lock and emit the unlock event.
                Self::execute_full_unlock(who);
            }
            Operation::Update(Effect::FullUnlock, _, who)
            | Operation::Unlock(Effect::FullUnlock, who) => {
                // Remove the schedule.
                <Schedules<T>>::remove(who);

                // Remove the lock and emit the unlock event.
                Self::execute_full_unlock(who);
            }
        }
    }

    /// Remove the lock and emit a [`Event::FullyUnlocked`] event.
    fn execute_full_unlock(who: &T::AccountId) {
        // Remove the balance lock.
        <CurrencyOf<T> as LockableCurrency<T::AccountId>>::remove_lock(T::LockId::get(), who);

        // Dispatch the event.
        Self::deposit_event(Event::FullyUnlocked { who: who.clone() });
    }

    /// Set the lock and emit a [`Event::PartiallyUnlocked`] event.
    fn execute_partial_unlock(who: &T::AccountId, balance_left_under_lock: BalanceOf<T>) {
        // Set the lock to the updated value.
        Self::set_lock(who, balance_left_under_lock);

        // Dispatch the event.
        Self::deposit_event(Event::PartiallyUnlocked {
            who: who.clone(),
            balance_left_under_lock,
        });
    }

    /// Set the lock.
    ///
    /// It is an implementation detail of [`Self::execute_partial_unlock`], but also used in tests.
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
