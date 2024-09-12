//! The benchmarks for the pallet.

// Allow float arithmetic in tests.
#![allow(clippy::float_arithmetic)]

use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use primitives_ethereum::{EcdsaSignature, EthereumAddress};

use crate::*;

/// The benchmark interface into the environment.
pub trait Interface: super::Config {
    /// Obtain an Account ID.
    ///
    /// This is an account to claim the ethereum address to.
    fn account_id_to_claim_to() -> <Self as frame_system::Config>::AccountId;

    /// Obtain an ethereum address.
    ///
    /// This is an ethereum account that is supposed to have a valid account claim associated with it
    /// in the runtime genesis.
    fn ethereum_address() -> EthereumAddress;

    /// Obtain an ECDSA signature that would fit the provided Account ID and the Ethereum address
    /// under the associated runtime.
    fn create_ecdsa_signature(
        account_id: &<Self as frame_system::Config>::AccountId,
        ethereum_address: &EthereumAddress,
    ) -> EcdsaSignature;
}

benchmarks! {
    where_clause {
        where
            T: Interface,
    }

    claim_account {
        let account_id = <T as Interface>::account_id_to_claim_to();
        let ethereum_address = <T as  Interface>::ethereum_address();
        let ethereum_signature = <T as  Interface>::create_ecdsa_signature(&account_id, &ethereum_address);

        // We assume the genesis doesn't have the corresponding claim; crash the bench if it does.
        assert!(Accounts::<T>::get(ethereum_address).is_none());
        assert!(EthereumAddresses::<T>::get(account_id.clone()).is_none());

        #[cfg(test)]
        let test_data = {
            use crate::mock;

            let mock_runtime_guard = mock::runtime_lock();

            let signed_claim_verifier_ctx = mock::MockSignedClaimVerifier::verify_context();
            signed_claim_verifier_ctx
                .expect()
                .times(1..)
                .return_const(Some(ethereum_address));

            (mock_runtime_guard, signed_claim_verifier_ctx)
        };

        let origin = RawOrigin::Signed(account_id.clone());

    }: _(origin, ethereum_address, ethereum_signature)
    verify {
        assert_eq!(Accounts::<T>::get(ethereum_address), Some(account_id.clone()));
        assert_eq!(EthereumAddresses::<T>::get(account_id), Some(ethereum_address));

        #[cfg(test)]
        {
            let (mock_runtime_guard, signed_claim_verifier_ctx) = test_data;

            signed_claim_verifier_ctx.checkpoint();

            drop(mock_runtime_guard);
        }
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test,
    );
}

#[cfg(test)]
impl Interface for crate::mock::Test {
    fn account_id_to_claim_to() -> <Self as frame_system::Config>::AccountId {
        10
    }

    fn ethereum_address() -> EthereumAddress {
        mock::eth(mock::EthAddr::New)
    }

    fn create_ecdsa_signature(
        _account_id: &<Self as frame_system::Config>::AccountId,
        _ethereum_address: &EthereumAddress,
    ) -> EcdsaSignature {
        EcdsaSignature::default()
    }
}
