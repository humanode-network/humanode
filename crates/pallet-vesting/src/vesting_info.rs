//! A substrate module to enforce private fields on `VestingInfo`.

use super::*;
use crate::traits::LinearUnlocking as LinearUnlockingT;

/// Struct to encode the vesting schedule of an individual account.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct VestingInfo<Balance, Moment> {
    /// Locked amount at genesis.
    locked: Balance,
    /// Starting moment for unlocking(vesting).
    start: Moment,
}

impl<Balance, Moment> VestingInfo<Balance, Moment>
where
    Balance: AtLeast32BitUnsigned + Copy,
    Moment: AtLeast32Bit + Copy + Bounded,
{
    /// Instantiate a new `VestingInfo`.
    pub fn new(locked: Balance, start: Moment) -> VestingInfo<Balance, Moment> {
        VestingInfo { locked, start }
    }

    /// Validate parameters for `VestingInfo`. Note that this does not check
    /// against `MinVestedTransfer`.
    pub fn is_valid(&self) -> bool {
        !self.locked.is_zero()
    }

    /// Locked amount at schedule creation.
    pub fn locked(&self) -> Balance {
        self.locked
    }

    /// Starting moment for unlocking(vesting).
    pub fn start(&self) -> Moment {
        self.start
    }

    /// Amount locked at moment.
    pub fn locked_at<LinearUnlocking: LinearUnlockingT<Balance, Moment>>(
        &self,
        moment: Moment,
    ) -> Balance {
        LinearUnlocking::locked_at(self.start, self.locked, moment)
    }

    /// Moment at which the schedule ends.
    pub fn end<LinearUnlocking: LinearUnlockingT<Balance, Moment>>(&self) -> Moment {
        LinearUnlocking::end(self.start, self.locked)
    }
}
