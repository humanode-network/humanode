//! Vesting related traits.

use frame_support::{dispatch::DispatchResult, traits::Currency};

pub trait LinearUnlocking {
    type Balance;
    type Moment;
    fn locked_at(&self, moment: Self::Moment) -> Self::Balance;
    fn end(&self) -> Self::Moment;
}

/// A vesting schedule over a currency. This allows a particular currency to have vesting limits
/// applied to it.
pub trait VestingSchedule<AccountId> {
    /// The quantity used to denote time; usually just a `BlockNumber`.
    type Moment;

    /// The currency that this schedule applies to.
    type Currency: Currency<AccountId>;

    /// Get the amount that is currently being vested and cannot be transferred out of this account.
    /// Returns `None` if the account has no vesting schedule.
    fn vesting_balance(who: &AccountId)
        -> Option<<Self::Currency as Currency<AccountId>>::Balance>;

    /// Adds a vesting schedule to a given account.
    ///
    /// If the account has `MaxVestingSchedules`, an Error is returned and nothing
    /// is updated.
    ///
    /// Is a no-op if the amount to be vested is zero.
    ///
    /// NOTE: This doesn't alter the free balance of the account.
    fn add_vesting_schedule(
        who: &AccountId,
        locked: <Self::Currency as Currency<AccountId>>::Balance,
        step: Self::Moment,
        per_step: <Self::Currency as Currency<AccountId>>::Balance,
        start: Self::Moment,
    ) -> DispatchResult;

    /// Checks if `add_vesting_schedule` would work against `who`.
    fn can_add_vesting_schedule(
        who: &AccountId,
        locked: <Self::Currency as Currency<AccountId>>::Balance,
        step: Self::Moment,
        per_step: <Self::Currency as Currency<AccountId>>::Balance,
        start: Self::Moment,
    ) -> DispatchResult;

    /// Remove a vesting schedule for a given account.
    ///
    /// NOTE: This doesn't alter the free balance of the account.
    fn remove_vesting_schedule(who: &AccountId, schedule_index: u32) -> DispatchResult;
}
