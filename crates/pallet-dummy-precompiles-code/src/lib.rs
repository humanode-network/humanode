//! A substrate pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::StorageVersion;
pub use pallet::*;
use sp_core::H160;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

/// The current precompiles addresses creating version.
pub const CURRENT_PRECOMPILES_ADDRESSES_CREATING_VERSION: u16 = 1;

/// The dummy code used to be stored for precompiles addresses.
const DUMMY_CODE: &str = "DUMMY_CODE";

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;

    use super::*;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configuration trait of this pallet.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_evm::Config {
        /// The list of precompiles adresses to be stored at evm with dummy code.
        type PrecompilesAddresses: Get<Vec<H160>>;
    }

    #[pallet::genesis_config]
    #[derive(Default)]
    pub struct GenesisConfig {}

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            for precompile_address in &T::PrecompilesAddresses::get() {
                pallet_evm::Pallet::<T>::create_account(
                    *precompile_address,
                    DUMMY_CODE.as_bytes().to_vec(),
                );
            }
        }
    }
}
