//! A substrate pallet containing the Native to EVM token swap integration.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::fungible::Inspect;
pub use pallet::*;
pub use weights::*;

pub mod weights;

/// Utility alias for easy access to the [`Inspect::Balance`] of the [`Config::NativeToken`] type.
type NativeBalanceOf<T> =
    <<T as Config>::NativeToken as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

/// Utility alias for easy access to the [`Inspect::Balance`] of the [`Config::EvmToken`] type.
type EvmBalanceOf<T> = <<T as Config>::EvmToken as Inspect<<T as Config>::EvmAccountId>>::Balance;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use fp_ethereum::ValidatedTransaction;
    use frame_support::{
        pallet_prelude::*,
        sp_runtime::traits::{Convert, UniqueSaturatedInto},
        storage::with_storage_layer,
        traits::{
            fungible::Mutate,
            tokens::{Preservation, Provenance},
        },
    };
    use frame_system::pallet_prelude::*;
    use sp_core::{H160, H256, U256};

    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configuration trait of this pallet.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_ethereum::Config {
        /// Overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The EVM user account identifier type.
        type EvmAccountId: Parameter + Into<H160>;

        /// Native token.
        type NativeToken: Inspect<Self::AccountId> + Mutate<Self::AccountId>;

        /// EVM token.
        type EvmToken: Inspect<Self::EvmAccountId>;

        /// The converter to determine how the balance amount should be converted from native
        /// to EVM token.
        type BalanceConverter: Convert<NativeBalanceOf<Self>, EvmBalanceOf<Self>>;

        /// The bridge pot native account.
        type BridgePotNative: Get<Self::AccountId>;

        /// The bridge pot EVM account.
        type BridgePotEvm: Get<Self::EvmAccountId>;

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
            /// The corresponding transaction hash executed in EVM.
            evm_transaction_hash: H256,
        },
    }

    #[pallet::call(weight(<T as Config>::WeightInfo))]
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
            T::EvmToken::can_deposit(&to, estimated_swapped_balance, Provenance::Extant)
                .into_result()?;

            T::NativeToken::transfer(&who, &T::BridgePotNative::get(), amount, preservation)?;

            let evm_balance_to_be_deposited: u128 =
                estimated_swapped_balance.unique_saturated_into();

            let transaction = pallet_ethereum::Transaction::EIP1559(ethereum::EIP1559Transaction {
                chain_id: <T as pallet_evm::Config>::ChainId::get(),
                nonce: pallet_evm::Pallet::<T>::account_basic(&T::BridgePotEvm::get().into())
                    .0
                    .nonce,
                max_priority_fee_per_gas: 0.into(),
                max_fee_per_gas: 0.into(),
                gas_limit: <T as pallet_evm::Config>::config()
                    .gas_transaction_call
                    .into(), // simple transfer
                action: ethereum::TransactionAction::Call(to.clone().into()),
                value: U256::from(evm_balance_to_be_deposited),
                input: Default::default(),
                access_list: Default::default(),
                odd_y_parity: false,
                r: Default::default(),
                s: Default::default(),
            });

            let evm_transaction_hash = transaction.hash();

            let _post_info = pallet_ethereum::ValidatedTransaction::<T>::apply(
                T::BridgePotEvm::get().into(),
                transaction,
            )
            .map_err(|dispatch_error_with_post_info| dispatch_error_with_post_info.error)?;

            Self::deposit_event(Event::BalancesSwapped {
                from: who,
                withdrawed_amount: amount,
                to,
                deposited_amount: estimated_swapped_balance,
                evm_transaction_hash,
            });

            Ok(())
        }
    }
}
