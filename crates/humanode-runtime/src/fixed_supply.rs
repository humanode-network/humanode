//! The implementation of the various bits and pieces that we use throughout the system to ensure
//! the fixed supply.

use core::marker::PhantomData;

use frame_support::traits::{Currency, OnUnbalanced};

use super::*;

/// The corrected implementation of the [`pallet_evm::EVMCurrencyAdapter`].
pub struct EvmTransactionCharger<C, OU>(PhantomData<(C, OU)>);

impl<T, C, OU> pallet_evm::OnChargeEVMTransaction<T> for EvmTransactionCharger<C, OU>
where
    T: pallet_evm::Config,
    C: Currency<<T as frame_system::Config>::AccountId>,
    OU: OnUnbalanced<<C as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance>,
{
    type LiquidityInfo =
        Option<<C as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance>;

    fn withdraw_fee(who: &H160, fee: U256) -> Result<Self::LiquidityInfo, pallet_evm::Error<T>> {
        <pallet_evm::EVMCurrencyAdapter<C, OU> as pallet_evm::OnChargeEVMTransaction<T>>::withdraw_fee(who, fee)
    }

    fn correct_and_deposit_fee(
        who: &H160,
        corrected_fee: U256,
        base_fee: U256,
        already_withdrawn: Self::LiquidityInfo,
    ) -> Self::LiquidityInfo {
        <pallet_evm::EVMCurrencyAdapter<C, OU> as pallet_evm::OnChargeEVMTransaction<T>>::correct_and_deposit_fee(who, corrected_fee, base_fee, already_withdrawn)
    }

    fn pay_priority_fee(tip: Self::LiquidityInfo) {
        if let Some(tip) = tip {
            // This is a rewrite of the default EVM implementation that blantly mishandles
            // imbalances. By not following the imbalances discipline (i.e. using the wrong
            // function) the EVM implementation leads to the appearance of two mirroring opposite
            // imbalances - while they could've just do `resolve_creating` instead.
            // This is the correct rewrite of the same logic.

            use pallet_evm::AddressMapping;
            let account_id = <T as pallet_evm::Config>::AddressMapping::into_account_id(
                <pallet_evm::Pallet<T>>::find_author(),
            );
            C::resolve_creating(&account_id, tip);
        }
    }
}
