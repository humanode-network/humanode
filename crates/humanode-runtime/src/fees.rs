use super::*;

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
