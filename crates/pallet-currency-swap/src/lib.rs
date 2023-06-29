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

/// Utility alias for easy access to [`CurrencySwap::From`] type from a given config.
type FromCurrencyOf<T> = <<T as Config>::CurrencySwap as traits::CurrencySwap<
    <T as frame_system::Config>::AccountId,
    <T as Config>::AccountIdTo,
>>::From;

/// Utility alias for easy access to [`CurrencySwap::From::Balance`] type from a given config.
type FromBalanceOf<T> =
    <FromCurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Utility alias for easy access to [`CurrencySwap::To`] type from a given config.
type ToCurrencyOf<T> = <<T as Config>::CurrencySwap as traits::CurrencySwap<
    <T as frame_system::Config>::AccountId,
    <T as Config>::AccountIdTo,
>>::To;

/// Utility alias for easy access to [`CurrencySwap::To::Balance`] type from a given config.
type ToBalanceOf<T> = <ToCurrencyOf<T> as Currency<<T as Config>::AccountIdTo>>::Balance;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        storage::with_storage_layer,
        traits::{ExistenceRequirement, Imbalance, WithdrawReasons},
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
        type CurrencySwap: CurrencySwap<Self::AccountId, Self::AccountIdTo>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Balances were swapped.
        BalancesSwapped {
            /// The account id balances withdrawed from.
            from: T::AccountId,
            /// The withdrawed balances amount.
            withdrawed_amount: FromBalanceOf<T>,
            /// The account id balances deposited to.
            to: T::AccountIdTo,
            /// The deposited balances amount.
            deposited_amount: ToBalanceOf<T>,
        },
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

            with_storage_layer(move || {
                let withdrawed_imbalance = FromCurrencyOf::<T>::withdraw(
                    &who,
                    amount,
                    WithdrawReasons::TRANSFER,
                    ExistenceRequirement::AllowDeath,
                )?;
                let withdrawed_amount = withdrawed_imbalance.peek();

                let deposited_imbalance =
                    T::CurrencySwap::swap(withdrawed_imbalance).map_err(Into::into)?;
                let deposited_amount = deposited_imbalance.peek();

                ToCurrencyOf::<T>::resolve_creating(&to, deposited_imbalance);

                Self::deposit_event(Event::BalancesSwapped {
                    from: who,
                    withdrawed_amount,
                    to,
                    deposited_amount,
                });

                Ok(())
            })
        }
    }
}
