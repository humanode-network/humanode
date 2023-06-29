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

/// The currency to convert from (from a given config).
type CurrencyFromOf<T> = <<T as Config>::CurrencySwap as traits::CurrencySwap<
    <T as frame_system::Config>::AccountId,
    <T as Config>::AccountIdTo,
>>::From;

/// The currency balance to convert from (from a given config).
type BalanceFromOf<T> =
    <CurrencyFromOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// The currency to convert to (from a given config).
type CurrencyToOf<T> = <<T as Config>::CurrencySwap as traits::CurrencySwap<
    <T as frame_system::Config>::AccountId,
    <T as Config>::AccountIdTo,
>>::To;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::{ExistenceRequirement, WithdrawReasons},
    };
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
        /// The user account identifier type to convert to.
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
            to: T::AccountIdTo,
            amount: BalanceFromOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let from_imbalance = CurrencyFromOf::<T>::withdraw(
                &who,
                amount,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::AllowDeath,
            )?;

            let to_imbalance = T::CurrencySwap::swap(from_imbalance).map_err(Into::into)?;

            CurrencyToOf::<T>::resolve_creating(&to, to_imbalance);

            Ok(())
        }
    }
}
