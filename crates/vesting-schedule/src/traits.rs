//! Generic vesting schedule related traits to abstract away the implementations.

use frame_support::traits::Currency;

/// [`VestingSchedule`] defines logic for currency vesting(unlocking).
pub trait VestingSchedule<AccountId> {
    /// The type used to denote time: Timestamp, BlockNumber, etc.
    type Moment;
    /// The currency that this schedule applies to.
    type Currency: Currency<AccountId>;
    /// An error that can occur at vesting schedule logic.
    type Error;
    /// Validate the schedule.
    fn validate(&self) -> Result<(), Self::Error>;
    /// Locked amount at provided moment.
    fn locked_at(
        &self,
        genesis_locked: <Self::Currency as Currency<AccountId>>::Balance,
        start: Self::Moment,
        moment: Self::Moment,
    ) -> <Self::Currency as Currency<AccountId>>::Balance;
    /// Moment at which the schedule ends.
    fn end(
        &self,
        genesis_locked: <Self::Currency as Currency<AccountId>>::Balance,
        start: Self::Moment,
    ) -> Self::Moment;
}
