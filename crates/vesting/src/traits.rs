//! Generic vesting related traits to abstract away the implementations.

use frame_support::{dispatch::DispatchResult, traits::Currency};
use vesting_schedule::VestingSchedule;

/// A general vesting logic.
pub trait Vesting<AccountId> {
    /// Defines logic of vesting schedule to be used.
    type VestingSchedule: VestingSchedule<AccountId>;

    /// Get the amount that is currently being vested and cannot be transferred out of this account.
    /// Returns `None` if the account has no vesting schedule.
    fn vesting_balance(
        who: &AccountId,
    ) -> Option<
        <<Self::VestingSchedule as VestingSchedule<AccountId>>::Currency as Currency<AccountId>>::Balance,
    >;

    /// Adds a vesting schedule to a given account.
    ///
    /// If the account has `MaxVestingSchedules`, an Error is returned and nothing
    /// is updated.
    ///
    /// Is a no-op if the amount to be vested is zero.
    fn add_vesting_schedule(
        who: &AccountId,
        locked: <<Self::VestingSchedule as VestingSchedule<AccountId>>::Currency as Currency<
            AccountId,
        >>::Balance,
        start: <Self::VestingSchedule as VestingSchedule<AccountId>>::Moment,
        vesting_schedule: Self::VestingSchedule,
    ) -> DispatchResult;

    /// Checks if `add_vesting_schedule` would work against `who`.
    fn can_add_vesting_schedule(
        who: &AccountId,
        locked: <<Self::VestingSchedule as VestingSchedule<AccountId>>::Currency as Currency<
            AccountId,
        >>::Balance,
        start: <Self::VestingSchedule as VestingSchedule<AccountId>>::Moment,
        vesting_schedule: Self::VestingSchedule,
    ) -> DispatchResult;

    /// Remove a vesting schedule for a given account.
    fn remove_vesting_schedule(who: &AccountId, schedule_index: u32) -> DispatchResult;
}
