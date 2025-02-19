//! A substrate pallet containing the native to evm currency swap integration.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::fungible::Inspect;
pub use pallet::*;
pub use weights::*;

pub mod weights;

/// Utility alias for easy access to the [`Inspect::Balance`] of the [`Config::NativeCurrency`] type.
type NativeBalanceOf<T> =
    <<T as Config>::NativeCurrency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

/// Utility alias for easy access to the [`Inspect::Balance`] of the [`Config::EvmCurrency`] type.
type EvmBalanceOf<T> =
    <<T as Config>::EvmCurrency as Inspect<<T as Config>::EvmAccountId>>::Balance;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        sp_runtime::traits::{Convert, MaybeDisplay},
        sp_std::fmt::Debug,
        storage::with_storage_layer,
        traits::{
            fungible::Mutate,
            tokens::{Preservation, Provenance},
        },
    };
    use frame_system::pallet_prelude::*;

    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configuration trait of this pallet.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The EVM user account identifier type.
        type EvmAccountId: Parameter
            + Member
            + MaybeSerializeDeserialize
            + Debug
            + MaybeDisplay
            + Ord
            + MaxEncodedLen;

        /// Native currency.
        type NativeCurrency: Inspect<Self::AccountId> + Mutate<Self::AccountId>;

        /// EVM currency.
        type EvmCurrency: Inspect<Self::EvmAccountId>;

        /// The converter to determine how the balance amount should be converted from one currency to
        /// another.
        type BalanceConverter: Convert<NativeBalanceOf<Self>, EvmBalanceOf<Self>>;

        /// The account to land the balances to when receiving the funds as part of the swap operation.
        type PotNativeBrige: Get<Self::AccountId>;

        /// The account to take the balances from when sending the funds as part of the swap operation.
        type PotEvmBridge: Get<Self::EvmAccountId>;

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
            withdrawed_amount: NativeBalanceOf<T>,
            /// The account id balances deposited to.
            to: T::EvmAccountId,
            /// The deposited balances amount.
            deposited_amount: EvmBalanceOf<T>,
        },
    }

    #[pallet::call(weight(T::WeightInfo))]
    impl<T: Config> Pallet<T> {
        /// Swap balances.
        #[pallet::call_index(0)]
        pub fn swap(
            origin: OriginFor<T>,
            to: T::EvmAccountId,
            #[pallet::compact] amount: NativeBalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            with_storage_layer(move || {
                Self::do_swap(who, to, amount, Preservation::Expendable)?;

                Ok(())
            })
        }

        /// Same as the swap call, but with a check that the swap will not kill the origin account.
        #[pallet::call_index(1)]
        pub fn swap_keep_alive(
            origin: OriginFor<T>,
            to: T::EvmAccountId,
            #[pallet::compact] amount: NativeBalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            with_storage_layer(move || {
                Self::do_swap(who, to, amount, Preservation::Preserve)?;

                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        /// General swap balances implementation.
        pub fn do_swap(
            who: T::AccountId,
            to: T::EvmAccountId,
            amount: NativeBalanceOf<T>,
            preservation: Preservation,
        ) -> DispatchResult {
            let estimated_swapped_balance = T::BalanceConverter::convert(amount);
            T::EvmCurrency::can_deposit(&to, estimated_swapped_balance, Provenance::Extant)
                .into_result()?;

            T::NativeCurrency::transfer(&who, &T::PotNativeBrige::get(), amount, preservation)?;

            // TODO: do visible ethereum transaction.

            Self::deposit_event(Event::BalancesSwapped {
                from: who,
                withdrawed_amount: amount,
                to,
                deposited_amount: estimated_swapped_balance,
            });

            Ok(())
        }
    }
}
