//! The vesting pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::MaxEncodedLen;
use frame_support::{
    pallet_prelude::*,
    traits::{Currency, StorageVersion},
};
pub use pallet::*;
use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize};
use vesting_schedule::VestingSchedule;

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
        type Currency: Currency<Self::AccountId>;

        /// The vesting schedule to operate with.
        type VestingSchedule: VestingSchedule<Self::AccountId>;

        /// Type used for expressing moment.
        type Moment: Parameter
            + Default
            + AtLeast32BitUnsigned
            + Copy
            + MaybeSerializeDeserialize
            + MaxEncodedLen;

        /// The getter for the current moment.
        type CurrentMoment: CurrentMoment<Self::Moment>;

        /// Maximum number of vesting schedules an account may have at a given moment.
        const MAX_VESTING_SCHEDULES: u32;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);
}
