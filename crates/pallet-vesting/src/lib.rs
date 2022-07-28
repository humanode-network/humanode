//! The vesting pallet.

#![cfg_attr(not(feature = "std"), no_std)]

mod moment;
mod vesting_driver;
mod vesting_driver_timestamp;
mod vesting_schedule;

use codec::MaxEncodedLen;
use frame_support::{
    pallet_prelude::*,
    storage::bounded_vec::BoundedVec,
    traits::{Currency, LockIdentifier, LockableCurrency, StorageVersion, WithdrawReasons},
};
pub use pallet::*;
use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, Zero};
use vesting_schedule::VestingSchedule;

/// Balance type alias.
type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

/// Provides the capability to get current moment.
pub trait CurrentMoment<Moment> {
    /// Return current moment.
    fn now() -> Moment;
}

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The currency to operate with.
        type Currency: LockableCurrency<Self::AccountId>;

        /// Type used for expressing moment.
        type Moment: Parameter
            + Default
            + AtLeast32BitUnsigned
            + Copy
            + MaybeSerializeDeserialize
            + MaxEncodedLen;

        /// The vesting schedule type to operate with.
        type VestingSchedule: VestingSchedule<
            Self::AccountId,
            Moment = Self::Moment,
            Currency = Self::Currency,
        >;

        /// The vesting schedule value itself.
        type VestinScheduleValue: Get<Self::VestingSchedule>;

        /// Maximum number of vesting schedules an account may have at a given moment.
        type MaxVestingSchedules: Get<u32>;

        /// An lock identifier for a lockable currency to be used in vesting.
        type VestingLockId: Get<LockIdentifier>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);
}
