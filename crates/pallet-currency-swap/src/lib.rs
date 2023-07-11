//! A substrate pallet containing the currency swap integration.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::fungible::{self, Balanced};
pub use pallet::*;
use primitives_currency_swap::CurrencySwap as CurrencySwapT;
pub use weights::*;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Utility alias for easy access to [`primitives_currency_swap::CurrencySwap::From`] type from a given config.
type FromFungibleOf<T> = <<T as Config>::CurrencySwap as CurrencySwapT<
    <T as frame_system::Config>::AccountId,
    <T as Config>::AccountIdTo,
>>::From;

/// Utility alias for easy access to the [`Currency::Balance`] of
/// the [`primitives_currency_swap::CurrencySwap::From`] type.
type FromBalanceOf<T> =
    <FromFungibleOf<T> as fungible::Inspect<<T as frame_system::Config>::AccountId>>::Balance;

/// Utility alias for easy access to [`primitives_currency_swap::CurrencySwap::To`] type from a given config.
type ToFungibleOf<T> = <<T as Config>::CurrencySwap as CurrencySwapT<
    <T as frame_system::Config>::AccountId,
    <T as Config>::AccountIdTo,
>>::To;

/// Utility alias for easy access to the [`Currency::Balance`] of
/// the [`primitives_currency_swap::CurrencySwap::To`] type.
type ToBalanceOf<T> = <ToFungibleOf<T> as fungible::Inspect<<T as Config>::AccountIdTo>>::Balance;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        storage::with_storage_layer,
        traits::{ExistenceRequirement, Imbalance},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::MaybeDisplay;
    use sp_std::fmt::Debug;

    use super::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configuration trait of this pallet.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The user account identifier type to convert to.
        type AccountIdTo: Parameter
            + Member
            + MaybeSerializeDeserialize
            + Debug
            + MaybeDisplay
            + Ord
            + MaxEncodedLen;

        /// Interface into currency swap implementation.
        type CurrencySwap: CurrencySwapT<Self::AccountId, Self::AccountIdTo>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Balances were swapped.
        BalancesSwapped {
            /// The Account ID the swapped funds are withdrawn from.
            from: T::AccountId,
            /// The amount of funds withdrawn for the swap.
            withdrawn_amount: FromBalanceOf<T>,
            /// The Account ID the swapped funds are deposited to.
            to: T::AccountIdTo,
            /// The deposited balances amount.
            deposited_amount: ToBalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The to account would not exist after the swap.
        ExistentialDeposit,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Swap balances.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::swap())]
        pub fn swap(
            origin: OriginFor<T>,
            to: T::AccountIdTo,
            amount: FromBalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_swap(who, to, amount, ExistenceRequirement::AllowDeath)?;
            Ok(())
        }

        /// Same as the swap call, but with a check that the swap will not kill the origin account.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::swap_keep_alive())]
        pub fn swap_keep_alive(
            origin: OriginFor<T>,
            to: T::AccountIdTo,
            amount: FromBalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_swap(who, to, amount, ExistenceRequirement::KeepAlive)?;
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// General swap balances implementation.
        pub fn do_swap(
            who: T::AccountId,
            to: T::AccountIdTo,
            amount: FromBalanceOf<T>,
            _existence_requirement: ExistenceRequirement,
        ) -> DispatchResult {
            with_storage_layer(move || {
                let from_credit = FromFungibleOf::<T>::withdraw(&who, amount)?;
                let from_amount = from_credit.peek();

                // It is fine to just return an error here without a proper cleanup
                // since we are at the [`with_storage_layer`].
                let to_credit =
                    T::CurrencySwap::swap(from_credit).map_err(|error| error.cause.into())?;
                let to_amount = to_credit.peek();

                if let Err(_to_credit) = ToFungibleOf::<T>::resolve(&to, to_credit) {
                    // It is fine to just return an error here without a proper cleanup
                    // since we are at the [`with_storage_layer`].
                    return Err(<Error<T>>::ExistentialDeposit.into());
                }

                Self::deposit_event(Event::BalancesSwapped {
                    from: who,
                    withdrawn_amount: from_amount,
                    to,
                    deposited_amount: to_amount,
                });

                Ok(())
            })
        }
    }
}
