//! The benchmarks for the pallet.

use frame_benchmarking::benchmarks;
use frame_support::{assert_ok, dispatch::DispatchResult};
use frame_system::RawOrigin;
use primitives_ethereum::{EcdsaSignature, EthereumAddress};

use crate::*;

/// The benchmarking extension for the vesting interface.
pub trait VestingInterface: traits::VestingInterface {
    /// The data to be passed from `prepare` to `verify`.
    type Data;

    /// Prepare vesting interface environment.
    fn prepare() -> Self::Data;
    /// Verify vesting interface environment,
    fn verify(data: Self::Data) -> DispatchResult;
}

/// The benchmark interface into the environment.
pub trait Interface: super::Config {
    /// Obtain an Account ID.
    ///
    /// This is an account to claim the funds to.
    fn account_id_to_claim_to() -> <Self as frame_system::Config>::AccountId;

    /// Obtain an ethereum address.
    ///
    /// This is an ethereum account that is supposed to have a valid calim associated with it
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
            <T as super::Config>::VestingInterface: VestingInterface,
    }

    claim {
        let account_id = <T as Interface>::account_id_to_claim_to();
        let ethereum_address = <T as  Interface>::ethereum_address();
        let ethereum_signature = <T as  Interface>::create_ecdsa_signature(&account_id, &ethereum_address);

        // We assume the genesis has the corresponding claim; crash the bench if it doesn't.
        let claim_info = Claims::<T>::get(ethereum_address).unwrap();

        let account_balance_before = <CurrencyOf<T>>::total_balance(&account_id);
        let currency_total_issuance_before = <CurrencyOf<T>>::total_issuance();

        #[cfg(test)]
        let test_data = {
            use crate::mock;

            let mock_runtime_guard = mock::runtime_lock();

            let recover_signer_ctx = mock::MockEthereumSignatureVerifier::recover_signer_context();
            recover_signer_ctx.expect().times(1..).return_const(Some(ethereum_address));

            (mock_runtime_guard, recover_signer_ctx)
        };

        let vesting = <T as super::Config>::VestingInterface::prepare();

        let origin = RawOrigin::Signed(account_id.clone());

    }: _(origin, ethereum_address, ethereum_signature)
    verify {
        assert_eq!(Claims::<T>::get(ethereum_address), None);

        let account_balance_after = <CurrencyOf<T>>::total_balance(&account_id);
        assert_eq!(account_balance_after - account_balance_before, claim_info.balance);
        assert_eq!(
            currency_total_issuance_before,
            <CurrencyOf<T>>::total_issuance(),
        );

        assert_ok!(<T as super::Config>::VestingInterface::verify(vesting));

        #[cfg(test)]
        {
            let (mock_runtime_guard, recover_signer_ctx) = test_data;

            recover_signer_ctx.checkpoint();

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
        42
    }

    fn ethereum_address() -> EthereumAddress {
        mock::eth(mock::EthAddr::Existing)
    }

    fn create_ecdsa_signature(
        _account_id: &<Self as frame_system::Config>::AccountId,
        _ethereum_address: &EthereumAddress,
    ) -> EcdsaSignature {
        EcdsaSignature::default()
    }
}

#[cfg(test)]
impl VestingInterface for <crate::mock::Test as super::Config>::VestingInterface {
    type Data = mock::__mock_MockVestingInterface_VestingInterface::__lock_under_vesting::Context;

    fn prepare() -> Self::Data {
        let lock_under_vesting_ctx = mock::MockVestingInterface::lock_under_vesting_context();
        lock_under_vesting_ctx
            .expect()
            .times(1..)
            .return_const(Ok(()));

        lock_under_vesting_ctx
    }

    fn verify(lock_under_vesting_ctx: Self::Data) -> DispatchResult {
        lock_under_vesting_ctx.checkpoint();
        Ok(())
    }
}
