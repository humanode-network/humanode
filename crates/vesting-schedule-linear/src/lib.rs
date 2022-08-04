//! The linear schedule for vesting.

#![cfg_attr(not(feature = "std"), no_std)]

use num_traits::{CheckedSub, Unsigned, Zero};

pub mod traits;

use traits::FracScale;

/// The linear schedule.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    codec::Encode,
    codec::Decode,
    codec::MaxEncodedLen,
    scale_info::TypeInfo,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct LinearSchedule<Balance, Duration> {
    /// The balance to lock.
    pub balance: Balance,
    /// The cliff duration (counting from the starting point).
    pub cliff: Duration,
    /// The vesting duration (counting from after the cliff).
    pub vesting: Duration,
}

impl<Balance, Duration> LinearSchedule<Balance, Duration>
where
    Balance: Unsigned + Copy,
    Duration: PartialOrd + Unsigned + CheckedSub + Copy,
{
    /// Compute the amount of balance to lock at any given point in the schedule
    /// specified by `duration_since_starting_point`.
    pub fn compute_locked_balance<S>(
        &self,
        duration_since_starting_point: Duration,
    ) -> Option<Balance>
    where
        S: FracScale<Value = Balance, FracPart = Duration>,
    {
        let progress = match duration_since_starting_point.checked_sub(&self.cliff) {
            // We don't have the progress yet because the cliff period did not pass yet, so
            // lock the whole balance.
            None => return Some(self.balance),
            Some(v) => v,
        };

        let locked_fraction = match self.vesting.checked_sub(&progress) {
            // We don't have the locked fraction already because the vesting period is already
            // over.
            // We guarantee that we unlock everything by making it so
            None => return Some(Zero::zero()),
            Some(v) => v,
        };

        S::frac_scale(&self.balance, &locked_fraction, &self.vesting)
    }
}
