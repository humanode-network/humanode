//! A substrate module to enforce private fields on `VestingInfo`.

use super::*;

/// Struct to encode the vesting schedule of an individual account.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct VestingInfo<Balance, Moment> {
    /// Locked amount at genesis.
    locked: Balance,
    /// Step moment between unlocking(vesting).
    step: Moment,
    /// Amount that gets unlocked every step moment after `starting_moment`.
    per_step: Balance,
    /// Starting moment for unlocking(vesting).
    start: Moment,
}

impl<Balance, Moment> VestingInfo<Balance, Moment>
where
    Balance: AtLeast32BitUnsigned + Copy,
    Moment: AtLeast32Bit + Copy + Bounded,
{
    /// Instantiate a new `VestingInfo`.
    pub fn new(
        locked: Balance,
        step: Moment,
        per_step: Balance,
        start: Moment,
    ) -> VestingInfo<Balance, Moment> {
        VestingInfo {
            locked,
            step,
            per_step,
            start,
        }
    }

    /// Validate parameters for `VestingInfo`. Note that this does not check
    /// against `MinVestedTransfer`.
    pub fn is_valid(&self) -> bool {
        !self.locked.is_zero() && !self.raw_per_step().is_zero()
    }

    /// Locked amount at schedule creation.
    pub fn locked(&self) -> Balance {
        self.locked
    }

    /// Stem moment between unlocking.
    pub fn step(&self) -> Moment {
        self.step
    }

    /// Amount that gets unlocked every step moment after `start`. Corrects for `per_step` of 0.
    /// We don't let `per_step` be less than 1, or else the vesting will never end.
    /// This should be used whenever accessing `per_step` unless explicitly checking for 0 values.
    pub fn per_step(&self) -> Balance {
        self.per_step.max(One::one())
    }

    /// Get the unmodified `per_step_moment`. Generally should not be used, but is useful for
    /// validating `per_step_moment`.
    pub(crate) fn raw_per_step(&self) -> Balance {
        self.per_step
    }

    /// Starting moment for unlocking(vesting).
    pub fn start(&self) -> Moment {
        self.start
    }

    /// Amount locked at moment `m`.
    pub fn locked_at<MomentToBalance: Convert<Moment, Balance>>(&self, m: Moment) -> Balance {
        // Moment that count toward vesting;
        // saturating to 0 when m < starting_moment.
        let vested_time = m.saturating_sub(self.start);
        let vested_steps = vested_time
            .checked_div(&self.step)
            .unwrap_or_else(Zero::zero);
        let vested_steps = MomentToBalance::convert(vested_steps);
        // Return amount that is still locked in vesting.
        vested_steps
            .checked_mul(&self.per_step()) // `per_block` accessor guarantees at least 1.
            .map(|to_unlock| self.locked.saturating_sub(to_unlock))
            .unwrap_or_else(Zero::zero)
    }

    // /// Moment at which the schedule ends (as type `Balance`).
    // pub fn ending_moment_as_balance<MomentToBalance: Convert<Moment, Balance>>(&self) -> Balance {
    //     let steps = if self.per_step() >= self.locked {
    //         // If `per_step` is bigger than `locked`, the schedule will end
    //         // the step after starting.
    //         One::one()
    //     } else {
    //         self.locked / self.per_step()
    //             + if (self.locked % self.per_step()).is_zero() {
    //                 Zero::zero()
    //             } else {
    //                 // `per_step` does not perfectly divide `locked`, so we need an extra step to
    //                 // unlock some amount less than `per_step`.
    //                 One::one()
    //             }
    //     };

    //     let start_moment = MomentToBalance::convert(self.start);
    //     let duration = self.step.checked_mul(v)
    //     start_moment.saturating_add(duration)
    // }
}
