//! The benchmarks for the pallet.

use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use primitives_ethereum::{EcdsaSignature, EthereumAddress};

use crate::*;

/// The benchmark interface into the environment.
pub trait Interface {
    type Config: super::Config;

    fn create_ethereum_address() -> EthereumAddress;
    fn create_claim_info() -> super::ClaimInfoOf<Self::Config>;
    fn create_ecdsa_signature() -> EcdsaSignature;
    fn create_account_id() -> <Self::Config as frame_system::Config>::AccountId;
}

benchmarks! {
    where_clause {
        where
            T: Interface<Config = T>
    }

    claim {
        let ethereum_address = <T as Interface>::create_ethereum_address();
        let claim_info = <T as Interface>::create_claim_info();
        let signature = <T as Interface>::create_ecdsa_signature();
        let account_id = <T as Interface>::create_account_id();
        <Claims<T>>::insert(ethereum_address, claim_info);
    }: _(RawOrigin::Signed(account_id), ethereum_address, signature)
    verify {
        assert_eq!(Claims::<T>::get(ethereum_address), None);
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test,
    );
}

#[cfg(test)]
impl Interface for crate::mock::Test {
    type Config = Self;

    fn create_ethereum_address() -> EthereumAddress {
        EthereumAddress::default()
    }

    fn create_claim_info() -> crate::ClaimInfoOf<Self::Config> {
        crate::types::ClaimInfo {
            balance: 0,
            vesting: None,
        }
    }

    fn create_ecdsa_signature() -> EcdsaSignature {
        EcdsaSignature::default()
    }

    fn create_account_id() -> <Self::Config as frame_system::Config>::AccountId {
        0
    }
}
