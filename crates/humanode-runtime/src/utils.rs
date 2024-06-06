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

/// Compute the longest mortal [`sp_runtime::generic::Era`] from the current block, assuming we are
/// executing from the context of the provided runtime.
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

/// A module to encapsulate the helper type aliases.
mod transaction {
    type RuntimeCallOf<T> = <T as frame_system::Config>::RuntimeCall;
    type PublicOf<T> = <T as frame_system::offchain::SigningTypes>::Public;
    type SignatureOf<T> = <T as frame_system::offchain::SigningTypes>::Signature;
    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    type IndexOf<T> = <T as frame_system::Config>::Index;
    type OverarchingCallOf<T> =
        <T as frame_system::offchain::SendTransactionTypes<RuntimeCallOf<T>>>::OverarchingCall;
    type ExtrinsicOf<T> =
        <T as frame_system::offchain::SendTransactionTypes<RuntimeCallOf<T>>>::Extrinsic;
    type SignaturePayloadOf<T> =
        <ExtrinsicOf<T> as sp_runtime::traits::Extrinsic>::SignaturePayload;

    /// Take a runtime call, and the use the [`frame_system::offchain`] facilities to create
    /// a transaction from it.
    pub fn create_transaction<T, C>(
        call: RuntimeCallOf<T>,
        public: PublicOf<T>,
        account: AccountIdOf<T>,
        nonce: IndexOf<T>,
    ) -> Option<(OverarchingCallOf<T>, SignaturePayloadOf<T>)>
    where
        T: Send + Sync,
        T: frame_system::Config,
        T: frame_system::offchain::SendTransactionTypes<
            RuntimeCallOf<T>,
            OverarchingCall = RuntimeCallOf<T>,
        >,
        T: frame_system::offchain::CreateSignedTransaction<RuntimeCallOf<T>>,
        C: frame_system::offchain::AppCrypto<PublicOf<T>, SignatureOf<T>>,
    {
        T::create_transaction::<C>(call, public, account, nonce)
    }
}
pub use transaction::create_transaction;
