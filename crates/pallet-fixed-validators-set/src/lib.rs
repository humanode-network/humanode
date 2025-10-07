//! A substrate pallet for providing a fixed validators set.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::StorageVersion;

pub use self::pallet::*;
pub use self::weights::*;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod weights;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, storage::in_storage_layer};
    use frame_system::pallet_prelude::*;

    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The maximum number of validators to provide.
        type MaxValidators: Get<u32>;

        /// The type of the validator in the fixed validator set.
        type ValidatorId: Member + Parameter + MaybeSerializeDeserialize + MaxEncodedLen;

        /// The weight information provider type.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// A list of the validators.
    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> =
        StorageValue<_, BoundedVec<T::ValidatorId, T::MaxValidators>, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// The set of validators to initialize the chain with.
        pub validators: BoundedVec<T::ValidatorId, T::MaxValidators>,
    }

    // The build of genesis for the pallet.
    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            <Validators<T>>::put(&self.validators);
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The fixed validators set has been updated.
        SetUpdated,
    }

    #[pallet::call(weight(T::WeightInfo))]
    impl<T: Config> Pallet<T> {
        /// Unlock the vested balance according to the schedule.
        #[pallet::call_index(0)]
        pub fn update_set(
            origin: OriginFor<T>,
            new_set: BoundedVec<T::ValidatorId, T::MaxValidators>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            in_storage_layer(|| {
                Validators::<T>::set(new_set);
                Self::deposit_event(Event::SetUpdated);
                Ok(())
            })
        }
    }
}
