//! A substrate pallet containing the currency swap integration.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use weights::*;

pub mod weights;

pub mod traits;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Currency};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::MaybeDisplay;
    use sp_std::fmt::Debug;
    use traits::CurrencySwap;

    use super::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configuration trait of this pallet.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type AccountIdTo: Parameter
            + Member
            + MaybeSerializeDeserialize
            + Debug
            + MaybeDisplay
            + Ord
            + MaxEncodedLen;

        type CurrencySwap: CurrencySwap<Self::AccountId, Self::AccountIdTo>;

        type WeightInfo: WeightInfo;
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::swap())]
        pub fn swap(
            origin: OriginFor<T>,
            account_id_to: T::AccountIdTo,
            balance: <<T::CurrencySwap as CurrencySwap<T::AccountId, T::AccountIdTo>>::From as Currency<T::AccountId>>::Balance,
        ) -> DispatchResult {
            let account_id_from = ensure_signed(origin)?;

            let _ = T::CurrencySwap::swap(&account_id_from, &account_id_to, balance)
                .map_err(Into::into)?;
            Ok(())
        }
    }
}
