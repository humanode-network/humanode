//! A substrate pallet containing the currency swap integration.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Currency;
pub use pallet::*;
pub use weights::*;

pub mod weights;

pub mod traits;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// The currency type related to `From` of `CurrencySwap` interface.
type CurrencyFromOf<T> = <<T as Config>::CurrencySwap as traits::CurrencySwap<
    <T as frame_system::Config>::AccountId,
    <T as Config>::AccountIdTo,
>>::From;

/// The balance type related to `CurrencyFromOf`.
type BalanceFromOf<T> =
    <CurrencyFromOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
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
        /// The user account identifier type balances send to.
        type AccountIdTo: Parameter
            + Member
            + MaybeSerializeDeserialize
            + Debug
            + MaybeDisplay
            + Ord
            + MaxEncodedLen;

        /// Interface into currency swap implementation.
        type CurrencySwap: CurrencySwap<Self::AccountId, Self::AccountIdTo>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Swap balances.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::swap())]
        pub fn swap(
            origin: OriginFor<T>,
            account_id_to: T::AccountIdTo,
            balance: BalanceFromOf<T>,
        ) -> DispatchResult {
            let account_id_from = ensure_signed(origin)?;

            let _ = T::CurrencySwap::swap(&account_id_from, &account_id_to, balance)
                .map_err(Into::into)?;
            Ok(())
        }
    }
}
