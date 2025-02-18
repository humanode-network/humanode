//! A substrate pallet containing the currency swap integration.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{
    fungible::Inspect, tokens::Provenance, Currency, ExistenceRequirement,
};
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
type FromCurrencyOf<T> = <<T as Config>::CurrencySwap as CurrencySwapT<
    <T as frame_system::Config>::AccountId,
    <T as Config>::AccountIdTo,
>>::From;

/// Utility alias for easy access to the [`Currency::Balance`] of
/// the [`primitives_currency_swap::CurrencySwap::From`] type.
type FromBalanceOf<T> =
    <FromCurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Utility alias for easy access to the [`Currency::NegativeImbalance`] of
/// the [`primitives_currency_swap::CurrencySwap::From`] type.
type FromNegativeImbalanceOf<T> =
    <FromCurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

/// Utility alias for easy access to [`primitives_currency_swap::CurrencySwap::To`] type from a given config.
type ToCurrencyOf<T> = <<T as Config>::CurrencySwap as CurrencySwapT<
    <T as frame_system::Config>::AccountId,
    <T as Config>::AccountIdTo,
>>::To;

/// Utility alias for easy access to the [`Currency::Balance`] of
/// the [`primitives_currency_swap::CurrencySwap::To`] type.
type ToBalanceOf<T> = <ToCurrencyOf<T> as Currency<<T as Config>::AccountIdTo>>::Balance;

/// Utility alias for easy access to the [`Currency::NegativeImbalance`] of
/// the [`primitives_currency_swap::CurrencySwap::To`] type.
type ToNegativeImbalanceOf<T> =
    <ToCurrencyOf<T> as Currency<<T as Config>::AccountIdTo>>::NegativeImbalance;

/// TODO: docs.
pub trait WithdrawImbalanceToBeSwapped<AccountIdFrom, Balance, NegativeImbalance> {
    /// TODO: docs.
    fn do_withdraw(
        account_id_from: &AccountIdFrom,
        value: Balance,
        existence_requirement: ExistenceRequirement,
    ) -> Result<NegativeImbalance, sp_runtime::DispatchError>;
}

/// TODO: docs.
pub trait DepositSwappedImbalance<AccountIdTo, NegativeImbalance> {
    /// TODO: docs.
    fn do_deposit(account_id_to: &AccountIdTo, negative_imbalance: NegativeImbalance);
}

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, storage::with_storage_layer, traits::Imbalance};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::MaybeDisplay;
    use sp_std::fmt::Debug;

    use super::*;

    #[pallet::pallet]
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

        /// TODO: docs.
        type WithdrawImbalanceToBeSwapped: WithdrawImbalanceToBeSwapped<
            Self::AccountId,
            FromBalanceOf<Self>,
            FromNegativeImbalanceOf<Self>,
        >;

        /// Interface into currency swap implementation.
        type CurrencySwap: CurrencySwapT<Self::AccountId, Self::AccountIdTo>;

        /// TODO: docs.
        type DepositSwappedImbalance: DepositSwappedImbalance<
            Self::AccountIdTo,
            ToNegativeImbalanceOf<Self>,
        >;

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

    #[pallet::call(weight(T::WeightInfo))]
    impl<T: Config> Pallet<T> {
        /// Swap balances.
        #[pallet::call_index(0)]
        pub fn swap(
            origin: OriginFor<T>,
            to: T::AccountIdTo,
            #[pallet::compact] amount: FromBalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            with_storage_layer(move || {
                Self::do_swap(who, to, amount, ExistenceRequirement::AllowDeath)?;

                Ok(())
            })
        }

        /// Same as the swap call, but with a check that the swap will not kill the origin account.
        #[pallet::call_index(1)]
        pub fn swap_keep_alive(
            origin: OriginFor<T>,
            to: T::AccountIdTo,
            #[pallet::compact] amount: FromBalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            with_storage_layer(move || {
                Self::do_swap(who, to, amount, ExistenceRequirement::KeepAlive)?;

                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        /// General swap balances implementation.
        pub fn do_swap(
            who: T::AccountId,
            to: T::AccountIdTo,
            amount: FromBalanceOf<T>,
            existence_requirement: ExistenceRequirement,
        ) -> DispatchResult {
            let estimated_swapped_balance = T::CurrencySwap::estimate_swapped_balance(amount);
            ToCurrencyOf::<T>::can_deposit(&to, estimated_swapped_balance, Provenance::Extant)
                .into_result()?;

            let withdrawed_imbalance =
                T::WithdrawImbalanceToBeSwapped::do_withdraw(&who, amount, existence_requirement)?;

            let withdrawed_amount = withdrawed_imbalance.peek();

            let deposited_imbalance =
                T::CurrencySwap::swap(withdrawed_imbalance).map_err(|error| {
                    // Here we undo the withdrawal to avoid having a dangling imbalance.
                    FromCurrencyOf::<T>::resolve_creating(&who, error.incoming_imbalance);
                    error.cause.into()
                })?;
            let deposited_amount = deposited_imbalance.peek();

            T::DepositSwappedImbalance::do_deposit(&to, deposited_imbalance);

            Self::deposit_event(Event::BalancesSwapped {
                from: who,
                withdrawed_amount,
                to,
                deposited_amount,
            });

            Ok(())
        }
    }
}
