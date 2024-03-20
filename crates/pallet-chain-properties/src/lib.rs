//! A substrate minimal Pallet that stores chain properties in the runtime.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::StorageVersion;
pub use pallet::*;

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

    use super::*;

    /// The Chain Properties Pallet
    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// Configuration trait of this pallet.
    #[pallet::config]
    pub trait Config: frame_system::Config {}

    /// Ss58 Prefix.
    #[pallet::storage]
    #[pallet::getter(fn ss58_prefix)]
    pub type Ss58Prefix<T> = StorageValue<_, u16, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(Default)]
    pub struct GenesisConfig {
        /// Ss58 prefix.
        pub ss58_prefix: u16,
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            Ss58Prefix::<T>::put(self.ss58_prefix);
        }
    }
}
