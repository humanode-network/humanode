//! Fee calculation logic.
//!
//! This is a temporary one, until we have a more full featured version, which will likely
//! be implemented in a separate, dedicated crate (or crates).

use super::*;

/// An [`frame_support::weights::WeightToFee`] implementation that converts any amount of weight to
/// zero fee, effectively making it so we don't charge any fee per transaction at all.
pub type FreeWeight = frame_support::weights::ConstantMultiplier<Balance, ConstU128<0>>;

/// The implementation of [`pallet_evm::FeeCalculator`] that configures the min gas price to be
/// zero, effectively making the gas free.
/// With this, we are supposed to not change any fees for the EVM transactions.
pub struct FreeGas;

impl pallet_evm::FeeCalculator for FreeGas {
    fn min_gas_price() -> (U256, Weight) {
        (U256::zero(), 0u64)
    }
}

/// No not take any fee.
///
/// Provides the implementations of the transaction charging traits that don't withdraw any fee
/// no matter what the input parameters are.
pub struct NoFee;

impl pallet_transaction_payment::OnChargeTransaction<Runtime> for NoFee {
    type Balance = Balance;
    type LiquidityInfo = ();

    fn withdraw_fee(
        _who: &AccountId,
        _call: &Call,
        _info: &DispatchInfoOf<Call>,
        _fee: Self::Balance,
        _tip: Self::Balance,
    ) -> Result<Self::LiquidityInfo, TransactionValidityError> {
        Ok(())
    }

    fn correct_and_deposit_fee(
        _who: &AccountId,
        _dispatch_info: &DispatchInfoOf<Call>,
        _post_info: &PostDispatchInfoOf<Call>,
        _corrected_fee: Self::Balance,
        _tip: Self::Balance,
        _already_withdrawn: Self::LiquidityInfo,
    ) -> Result<(), TransactionValidityError> {
        Ok(())
    }
}

impl pallet_evm::OnChargeEVMTransaction<Runtime> for NoFee {
    type LiquidityInfo = ();

    fn withdraw_fee(
        _who: &H160,
        _fee: U256,
    ) -> Result<Self::LiquidityInfo, pallet_evm::Error<Runtime>> {
        Ok(())
    }

    fn correct_and_deposit_fee(
        _who: &H160,
        _corrected_fee: U256,
        _base_fee: U256,
        _already_withdrawn: Self::LiquidityInfo,
    ) -> Self::LiquidityInfo {
    }

    fn pay_priority_fee(_tip: Self::LiquidityInfo) {}
}
