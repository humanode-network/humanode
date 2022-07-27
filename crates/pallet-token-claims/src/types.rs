//! Custom types we use.

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{Deserialize, RuntimeDebug, Serialize};
use scale_info::TypeInfo;

/// The claim information.
#[derive(
    Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ClaimInfo<Balance, Vesting> {
    /// The amount to claim.
    pub balance: Balance,
    /// The vesting configuration for the given claim.
    pub vesting: Option<Vesting>,
}
