//! A pallet that captures the moment of the chain start.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{StorageVersion, Time};

pub use self::pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use super::*;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// A type representing the time moment and providing the current time.
        type Time: Time;
    }

    /// The captured chain start moment.
    #[pallet::storage]
    #[pallet::getter(fn chain_start)]
    pub type ChainStart<T> = StorageValue<_, <<T as Config>::Time as Time>::Moment, OptionQuery>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            if n != 1u8.into() {
                return Weight::zero();
            }
            <T as frame_system::Config>::DbWeight::get().writes(1)
        }

        fn on_finalize(n: BlockNumberFor<T>) {
            if n != 1u8.into() {
                return;
            }

            let now = T::Time::now();

            // Ensure that the chain start is properly initialized.
            assert_ne!(
                now,
                0u8.into(),
                "the chain start moment is zero, it is not right"
            );

            <ChainStart<T>>::put(now);
        }
    }
}
