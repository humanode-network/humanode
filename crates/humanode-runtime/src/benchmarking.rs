//! The benchmarking utilities.

use eip712_common::{EcdsaSignature, EthereumAddress};
use eip712_common_test_utils::{ecdsa_pair, ecdsa_sign, ethereum_address_from_seed};

use super::*;
use crate::dev_utils::*;

impl pallet_token_claims::benchmarking::Interface for Runtime {
    fn account_id_to_claim_to() -> <Self as frame_system::Config>::AccountId {
        account_id("Alice")
    }

    fn ethereum_address() -> EthereumAddress {
        ethereum_address_from_seed(b"Alice")
    }

    fn create_ecdsa_signature(
        account_id: &<Self as frame_system::Config>::AccountId,
        ethereum_address: &EthereumAddress,
    ) -> EcdsaSignature {
        if ethereum_address != &ethereum_address_from_seed(b"Alice") {
            panic!("bad ethereum address");
        }

        let chain_id: [u8; 32] = U256::from(crate::eip712::ETHEREUM_MAINNET_CHAIN_ID).into();
        let verifying_contract = crate::eip712::genesis_verifying_contract();
        let domain = eip712_common::Domain {
            name: "Humanode Token Claim",
            version: "1",
            chain_id: &chain_id,
            verifying_contract: &verifying_contract,
        };

        let pair = ecdsa_pair(b"Alice");
        let msg_hash = eip712_token_claim::make_message_hash(domain, account_id.as_ref());
        ecdsa_sign(&pair, &msg_hash)
    }
}

impl pallet_vesting::benchmarking::Interface for Runtime {
    fn account_id() -> <Self as frame_system::Config>::AccountId {
        account_id("Alice")
    }

    fn schedule() -> <Self as pallet_vesting::Config>::Schedule {
        use vesting_schedule_linear::LinearSchedule;
        vec![LinearSchedule {
            balance: 100,
            cliff: 10 * 24 * 60 * 60 * 1000,   // 10 days
            vesting: 10 * 24 * 60 * 60 * 1000, // 10 days
        }]
        .try_into()
        .unwrap()
    }
}
