//! A substrate pallet containing the EVM swap integration.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{
    fungible::{Inspect, Mutate},
    tokens::{Preservation, Provenance},
};
pub use pallet::*;
use sp_core::{Get, H160, U256};
pub use weights::*;

pub mod precompile;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Utility alias for easy access to the [`Inspect::Balance`] of the [`Config::NativeToken`] type.
pub type NativeBalanceOf<T> =
    <<T as Config>::NativeToken as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

/// Utility alias for easy access to the [`Inspect::Balance`] of the [`Config::EvmToken`] type.
pub type EvmBalanceOf<T> =
    <<T as Config>::EvmToken as Inspect<<T as Config>::EvmAccountId>>::Balance;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use fp_ethereum::ValidatedTransaction;
    use frame_support::{
        dispatch::PostDispatchInfo,
        pallet_prelude::*,
        sp_runtime::traits::{Convert, UniqueSaturatedInto},
        storage::with_storage_layer,
    };
    use frame_system::pallet_prelude::*;
    use pallet_evm::GasWeightMapping;
    use sp_core::H256;

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
        ///
        /// TODO(#1462): switch from `Mutate` to `Balanced` fungible interface.
        type NativeToken: Inspect<Self::AccountId> + Mutate<Self::AccountId>;

        /// EVM token.
        ///
        /// TODO(#1462): switch from `Mutate` to `Balanced` fungible interface.
        type EvmToken: Inspect<Self::EvmAccountId> + Mutate<Self::EvmAccountId>;

        /// The converter to determine how the balance amount should be converted from native
        /// to EVM token.
        type BalanceConverterNativeToEvm: Convert<NativeBalanceOf<Self>, EvmBalanceOf<Self>>;

        /// The converter to determine how the balance amount should be converted from EVM
        /// to native token.
        type BalanceConverterEvmToNative: Convert<EvmBalanceOf<Self>, NativeBalanceOf<Self>>;

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
            let estimated_swapped_balance = T::BalanceConverterNativeToEvm::convert(amount);
            T::EvmToken::can_deposit(&to, estimated_swapped_balance, Provenance::Extant)
                .into_result()?;

            T::EvmToken::can_withdraw(&T::BridgePotEvm::get(), estimated_swapped_balance)
                // Bridge pot EVM account shouldn't be killed.
                .into_result(true)?;

            T::NativeToken::transfer(&who, &T::BridgePotNative::get(), amount, preservation)?;

            let evm_transaction_hash = Self::execute_ethereum_transfer(
                T::BridgePotEvm::get().into(),
                to.clone().into(),
                estimated_swapped_balance.unique_saturated_into(),
            )?;

            Self::deposit_event(Event::BalancesSwapped {
                from: who,
                withdrawed_amount: amount,
                to,
                deposited_amount: estimated_swapped_balance,
                evm_transaction_hash,
            });

            Ok(())
        }

        /// Execute ethereum transfer from source address to target EVM address with provided
        /// balance to be sent.
        fn execute_ethereum_transfer(
            source_address: H160,
            target_address: H160,
            balance: u128,
        ) -> Result<H256, DispatchError> {
            let transaction =
                ethereum_transfer_transaction::<T>(source_address, target_address, balance);
            let transaction_hash = transaction.hash();

            let post_info =
                pallet_ethereum::ValidatedTransaction::<T>::apply(source_address, transaction)
                    .map_err(|dispatch_error_with_post_info| dispatch_error_with_post_info.error)?;

            debug_assert!(
                post_info
                    == PostDispatchInfo {
                        actual_weight: Some(
                            T::GasWeightMapping::gas_to_weight(
                                T::config().gas_transaction_call.unique_saturated_into(),
                                true,
                            )
                        ),
                        pays_fee: Pays::No
                    },
                "we must ensure that actual weight corresponds to gas used for simple transfer call"
            );

            Ok(transaction_hash)
        }
    }
}

/// A helper function to prepare simple ethereum transfer transaction.
pub(crate) fn ethereum_transfer_transaction<T: pallet_evm::Config>(
    source_address: H160,
    target_address: H160,
    balance: u128,
) -> pallet_ethereum::Transaction {
    pallet_ethereum::Transaction::EIP1559(ethereum::EIP1559Transaction {
        chain_id: T::ChainId::get(),
        nonce: pallet_evm::Pallet::<T>::account_basic(&source_address)
            .0
            .nonce,
        max_priority_fee_per_gas: 0.into(),
        max_fee_per_gas: 0.into(),
        gas_limit: T::config().gas_transaction_call.into(),
        action: ethereum::TransactionAction::Call(target_address),
        value: U256::from(balance),
        input: Default::default(),
        access_list: Default::default(),
        odd_y_parity: false,
        r: Default::default(),
        s: Default::default(),
    })
}
