//! A substrate minimal Pallet that stores ERC20 related data in the runtime.

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
    use frame_support::{
        pallet_prelude::*,
        sp_runtime::{traits::MaybeDisplay, FixedPointOperand},
        sp_std::fmt::Debug,
        traits::tokens::Balance,
    };

    use super::*;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configuration trait of this pallet.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The user account identifier type.
        type AccountId: Parameter
            + Member
            + MaybeSerializeDeserialize
            + Debug
            + MaybeDisplay
            + Ord
            + MaxEncodedLen;

        /// The balance of an account.
        type Balance: Balance
            + MaybeSerializeDeserialize
            + Debug
            + MaxEncodedLen
            + FixedPointOperand;
    }

    /// ERC20-style approvals data.
    /// (Owner => Allowed => Amount).
    #[pallet::storage]
    #[pallet::getter(fn approvals)]
    pub type Approvals<T> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        <T as Config>::AccountId,
        Blake2_128Concat,
        <T as Config>::AccountId,
        <T as Config>::Balance,
    >;
}
