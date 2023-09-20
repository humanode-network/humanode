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

/// The current precompiles addresses creation version.
pub const CURRENT_CREATION_VERSION: u16 = 1;

/// The dummy code used to be stored for precompiles addresses - 0x5F5FFD (as raw bytes).
///
/// This is actually a hand-crafted sequence of opcodes for a bare-bones revert.
/// The REVERT opcode (which is FD) - it takes two arguments from the stack with PUSH0 (5F twice).
pub const DUMMY_CODE: [u8; 3] = [95, 95, 253];

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, sp_std::vec::Vec};
    use frame_system::pallet_prelude::*;

    use super::*;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configuration trait of this pallet.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_evm::Config {
        /// The list of precompiles adresses to be created at evm with dummy code.
        type PrecompilesAddresses: Get<Vec<H160>>;
    }

    /// The creation version.
    #[pallet::storage]
    #[pallet::getter(fn creation_version)]
    pub type CreationVersion<T: Config> = StorageValue<_, u16, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(Default)]
    pub struct GenesisConfig {}

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            for precompile_address in &T::PrecompilesAddresses::get() {
                pallet_evm::Pallet::<T>::create_account(*precompile_address, DUMMY_CODE.to_vec());
            }
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            let creation_version = Self::creation_version();
            let mut weight = T::DbWeight::get().reads(1);

            if creation_version != CURRENT_CREATION_VERSION {
                for precompile_address in &T::PrecompilesAddresses::get() {
                    let code = pallet_evm::Pallet::<T>::account_codes(*precompile_address);
                    weight += T::DbWeight::get().reads(1);

                    if code != DUMMY_CODE.to_vec() {
                        pallet_evm::Pallet::<T>::create_account(
                            *precompile_address,
                            DUMMY_CODE.to_vec(),
                        );
                        weight += T::DbWeight::get().reads_writes(1, 1);
                    }
                }

                <CreationVersion<T>>::put(CURRENT_CREATION_VERSION);
            }

            weight
        }

        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
            // Do nothing.
            Ok(Vec::new())
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
            use sp_std::vec::Vec;

            let mut not_created_precompiles = Vec::new();

            for precompile_address in &T::PrecompilesAddresses::get() {
                let code = pallet_evm::Pallet::<T>::account_codes(*precompile_address);
                if code != DUMMY_CODE.to_vec() {
                    not_created_precompiles.push(*precompile_address);
                }
            }

            if !not_created_precompiles.is_empty() {
                return Err("precompiles not created properly: {:not_created_precompiles}");
            }

            Ok(())
        }
    }
}
