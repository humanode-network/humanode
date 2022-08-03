//! Custom types we use.

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::RuntimeDebug;
#[cfg(feature = "std")]
use frame_support::{Deserialize, Serialize};
use scale_info::TypeInfo;

/// The lock information.
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct LockInfo<Balance, Schedule> {
    /// The initial (total) balance locked under this lock.
    pub initial_locked_balance: Balance,
    /// The unlocking schedule.
    pub schedule: Schedule,
}
