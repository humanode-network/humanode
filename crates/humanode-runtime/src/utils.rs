use crate::GenericSignedExtra;

/// Returns longest possible era for the number of block hashes that are cached by the runtime.
///
/// Take the biggest period possible, considering the number of cached block hashes.
/// In the case of overflow, we pass default (`0`) and let `Era::mortal`
/// clamp the value to the lower bound
pub fn longest_era_for_block_hashes(
    current_block: u64,
    block_hash_count: u64,
) -> sp_runtime::generic::Era {
    let period: u64 = block_hash_count
        .checked_next_power_of_two()
        .map(|c| c / 2)
        .unwrap_or_default();
    sp_runtime::generic::Era::mortal(period, current_block)
}

/// Compute the longest mortal [`Era`] from the current block, assuming we are executing from
/// the context of the provided runtime.
pub fn current_era<T>() -> sp_runtime::generic::Era
where
    T: frame_system::Config,
    <T as frame_system::Config>::BlockNumber: Into<u64>,
{
    let current_block_number = frame_system::Pallet::<T>::block_number()
        .into()
        // `block_number` is initiated with `n+1`
        // so the actual block number is `n`.
        .saturating_sub(1);
    let block_hash_count = <<T as frame_system::Config>::BlockHashCount as sp_core::Get<
        <T as frame_system::Config>::BlockNumber,
    >>::get()
    .into();
    longest_era_for_block_hashes(current_block_number, block_hash_count)
}

pub type TransactionPaymentBalanceOf<T> = <<T as pallet_transaction_payment::Config>::OnChargeTransaction as pallet_transaction_payment::OnChargeTransaction<T>>::Balance;

/// Create the signed extra for any runtime.
pub fn create_extra<R>(
    nonce: <R as frame_system::Config>::Index,
    era: sp_runtime::generic::Era,
    tip: TransactionPaymentBalanceOf<R>,
) -> GenericSignedExtra<R>
where
    R: Send + Sync,
    R: frame_system::Config,
    R::RuntimeCall: frame_support::dispatch::Dispatchable<
        Info = frame_support::dispatch::DispatchInfo,
        PostInfo = frame_support::dispatch::PostDispatchInfo,
    >,
    R: pallet_transaction_payment::Config,
    TransactionPaymentBalanceOf<R>: Send + Sync + sp_runtime::FixedPointOperand,
    R: pallet_bioauth::Config,
    R: pallet_token_claims::Config,
{
    (
        frame_system::CheckSpecVersion::<R>::new(),
        frame_system::CheckTxVersion::<R>::new(),
        frame_system::CheckGenesis::<R>::new(),
        frame_system::CheckEra::<R>::from(era),
        frame_system::CheckNonce::<R>::from(nonce),
        frame_system::CheckWeight::<R>::new(),
        pallet_bioauth::CheckBioauthTx::<R>::new(),
        pallet_transaction_payment::ChargeTransactionPayment::<R>::from(tip),
        pallet_token_claims::CheckTokenClaim::<R>::new(),
    )
}
