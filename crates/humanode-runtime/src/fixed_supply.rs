//! The implementation of the various bits and pieces that we use throughout the system to ensure
//! the fixed supply.

use core::marker::PhantomData;

use frame_support::traits::{Currency, OnUnbalanced};
use sp_runtime::traits::UniqueSaturatedInto;

use super::*;

// /// The corrected implementation of the [`pallet_evm::EVMCurrencyAdapter`].
// pub struct EvmTransactionCharger<C, OU>(PhantomData<(C, OU)>);

// impl<T, C, OU> pallet_evm::OnChargeEVMTransaction<T> for EvmTransactionCharger<C, OU>
// where
//     T: pallet_evm::Config,
//     C: Currency<
//         <<T as pallet_evm::Config>::AccountProvider as pallet_evm::AccountProvider>::AccountId,
//     >,
//     OU: OnUnbalanced<
//         <C as Currency<
//             <<T as pallet_evm::Config>::AccountProvider as pallet_evm::AccountProvider>::AccountId,
//         >>::NegativeImbalance,
//     >,
//     U256: UniqueSaturatedInto<<C as Currency<<T as frame_system::Config>::AccountId>>::Balance>,
// {
//     type LiquidityInfo = Option<
//         <C as Currency<
//             <<T as pallet_evm::Config>::AccountProvider as pallet_evm::AccountProvider>::AccountId,
//         >>::NegativeImbalance,
//     >;

//     fn withdraw_fee(who: &H160, fee: U256) -> Result<Self::LiquidityInfo, pallet_evm::Error<T>> {
//         <pallet_evm::EVMCurrencyAdapter<C, OU> as pallet_evm::OnChargeEVMTransaction<T>>::withdraw_fee(who, fee)
//     }

//     fn correct_and_deposit_fee(
//         who: &H160,
//         corrected_fee: U256,
//         base_fee: U256,
//         already_withdrawn: Self::LiquidityInfo,
//     ) -> Self::LiquidityInfo {
//         <pallet_evm::EVMCurrencyAdapter<C, OU> as pallet_evm::OnChargeEVMTransaction<T>>::correct_and_deposit_fee(who, corrected_fee, base_fee, already_withdrawn)
//     }

//     fn pay_priority_fee(tip: Self::LiquidityInfo) {
//         if let Some(tip) = tip {
//             // Handle the tips in the same manner as the regular fee.
//             OU::on_unbalanced(tip);
//         }
//     }
// }
