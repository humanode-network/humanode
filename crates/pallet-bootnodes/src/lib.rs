//! A substrate pallet for storing the bootnodes for the validator set construction purposes.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::StorageVersion;
pub use pallet::*;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;

    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The maximum number of bootnodes storage value.
        type MaxBootnodes: Get<u32>;

        /// The type of the bootnode.
        type BootnodeId: Member + Parameter + MaybeSerializeDeserialize + MaxEncodedLen;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// A list of the bootnodes.
    #[pallet::storage]
    #[pallet::getter(fn bootnodes)]
    pub type Bootnodes<T: Config> =
        StorageValue<_, BoundedVec<T::BootnodeId, T::MaxBootnodes>, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// The list of bootnodes to use.
        pub bootnodes: BoundedVec<T::BootnodeId, T::MaxBootnodes>,
    }

    // The build of genesis for the pallet.
    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            <Bootnodes<T>>::put(&self.bootnodes);
        }
    }
}
