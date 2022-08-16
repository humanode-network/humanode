//! Signed extension implentation for token claims.

use core::marker::PhantomData;

use frame_support::{
    dispatch::Dispatchable,
    pallet_prelude::*,
    sp_runtime,
    traits::IsSubType,
    unsigned::{TransactionValidity, TransactionValidityError},
    weights::DispatchInfo,
};
use sp_runtime::traits::{DispatchInfoOf, SignedExtension};

use super::*;
use crate::{traits::verify_ethereum_signature, types::EthereumSignatureMessageParams};

impl<T: Config> Pallet<T> {
    /// Validate that the `claim` is correct and should be allowed for inclusion.
    ///
    /// Implement the flood protection logic.
    fn validate_claim_call(who: &T::AccountId, call: &Call<T>) -> TransactionValidity {
        // Check if the call matches.
        let (ethereum_address, ethereum_signature) = match call {
            // Allow `claim` call.
            Call::claim {
                ethereum_address,
                ethereum_signature,
            } => (ethereum_address, ethereum_signature),
            // Deny all unknown calls.
            _ => return Err(TransactionValidityError::Invalid(InvalidTransaction::Call)),
        };

        // Check the signature.
        let message_params = EthereumSignatureMessageParams {
            account_id: who.clone(),
            ethereum_address: *ethereum_address,
        };
        if !verify_ethereum_signature::<<T as Config>::EthereumSignatureVerifier>(
            ethereum_signature,
            &message_params,
            ethereum_address,
        ) {
            return Err(TransactionValidityError::Invalid(
                InvalidTransaction::BadProof,
            ));
        }

        // Check the presence of a claim.
        if !<Claims<T>>::contains_key(ethereum_address) {
            return Err(TransactionValidityError::Invalid(InvalidTransaction::Call));
        }

        // All good, letting through.
        Ok(ValidTransaction::default())
    }
}

/// Check the `claim` call for validity.
///
/// The call is free, so this check is required to ensure it will be properly verified to
/// prevent chain flooding.
#[derive(Clone, Eq, PartialEq, codec::Encode, codec::Decode, scale_info::TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CheckTokenClaim<T: Config + Send + Sync>(PhantomData<T>);

impl<T: Config + Send + Sync> SignedExtension for CheckTokenClaim<T>
where
    T::Call: Dispatchable<Info = DispatchInfo>,
    <T as frame_system::Config>::Call: IsSubType<Call<T>>,
{
    const IDENTIFIER: &'static str = "CheckTokenClaim";
    type AccountId = T::AccountId;
    type Call = T::Call;
    type AdditionalSigned = ();
    type Pre = ();

    fn additional_signed(
        &self,
    ) -> Result<Self::AdditionalSigned, frame_support::unsigned::TransactionValidityError> {
        Ok(())
    }

    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<Self::Pre, frame_support::unsigned::TransactionValidityError> {
        self.validate(who, call, info, len).map(|_| ())
    }

    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        match call.is_sub_type() {
            Some(call) => Pallet::<T>::validate_claim_call(who, call),
            _ => Ok(Default::default()),
        }
    }
}

impl<T: Config + Send + Sync> core::fmt::Debug for CheckTokenClaim<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "CheckTokenClaim")
    }
}

impl<T: Config + Send + Sync> CheckTokenClaim<T> {
    /// Create a new [`CheckTokenClaim`] instance.
    #[allow(clippy::new_without_default)] // following the pattern
    pub fn new() -> Self {
        Self(PhantomData)
    }
}
