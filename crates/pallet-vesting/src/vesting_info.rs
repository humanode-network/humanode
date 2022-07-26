//! VestingInfo.

use super::*;

/// Struct to encode the vesting schedule of an individual account.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct VestingInfo<Balance, Moment> {
    /// Locked amount at genesis.
    locked: Balance,
    /// Starting moment for unlocking(vesting).
    start: Moment,
}
