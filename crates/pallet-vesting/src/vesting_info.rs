//! A substrate module to enforce private fields on `VestingInfo`.

use super::*;
use crate::traits::LinearUnlocking as LinearUnlockingT;

/// Struct to encode the vesting schedule of an individual account.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct VestingInfo<Balance, Moment, LinearUnlocking> {
    /// Locked amount at genesis.
    locked: Balance,
    /// Starting moment for unlocking(vesting).
    start: Moment,
    /// Linear unlocking function.
    linear_unlocking: LinearUnlocking,
}

impl<Balance, Moment, LinearUnlocking> VestingInfo<Balance, Moment, LinearUnlocking>
where
    Balance: AtLeast32BitUnsigned + Copy,
    Moment: AtLeast32Bit + Copy + Bounded,
    LinearUnlocking: LinearUnlockingT,
{
    /// Instantiate a new `VestingInfo`.
    pub fn new(
        locked: Balance,
        start: Moment,
        linear_unlocking: LinearUnlocking,
    ) -> VestingInfo<Balance, Moment, LinearUnlocking> {
        VestingInfo {
            locked,
            start,
            linear_unlocking,
        }
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
    pub fn locked_at(&self, moment: Moment) -> Balance {
        self.linear_unlocking.locked_at(moment)
    }

    /// Moment at which the schedule ends.
    pub fn end(&self) -> Moment {
        self.linear_unlocking.end()
    }
}
