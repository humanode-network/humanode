//! The benchmarking utilities.

use eip712_common::{keccak_256, EcdsaSignature, EthereumAddress};

use super::*;

const ALICE: [u8; 32] =
    hex_literal::hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");

fn alice_secret() -> libsecp256k1::SecretKey {
    libsecp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
}

fn alice_ethereum_address() -> EthereumAddress {
    let mut ethereum_address = [0u8; 20];
    ethereum_address.copy_from_slice(
        &keccak_256(&libsecp256k1::PublicKey::from_secret_key(&alice_secret()).serialize()[1..65])
            [12..],
    );
    EthereumAddress(ethereum_address)
}

fn alice_sign(msg_hash: &[u8; 32]) -> EcdsaSignature {
    let (sig, recovery_id) =
        libsecp256k1::sign(&libsecp256k1::Message::parse(&msg_hash), &alice_secret());
    let mut ecdsa_signature = [0u8; 65];
    ecdsa_signature[0..64].copy_from_slice(&sig.serialize()[..]);
    ecdsa_signature[64] = recovery_id.serialize();
    EcdsaSignature(ecdsa_signature)
}

impl pallet_token_claims::benchmarking::Interface for Runtime {
    fn account_id_to_claim_to() -> <Self as frame_system::Config>::AccountId {
        AccountId::from(ALICE)
    }

    fn ethereum_address() -> EthereumAddress {
        alice_ethereum_address()
    }

    fn create_ecdsa_signature(
        account_id: &<Self as frame_system::Config>::AccountId,
        ethereum_address: &EthereumAddress,
    ) -> EcdsaSignature {
        if ethereum_address != &alice_ethereum_address() {
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

        let msg_hash = eip712_token_claim::make_message_hash(domain, account_id.as_ref());
        alice_sign(&msg_hash)
    }
}

impl pallet_token_claims::benchmarking::VestingInterface for vesting::TokenClaimsInterface {
    type Data = ();
    fn prepare() {}
    fn verify(_: ()) {}
}

impl pallet_vesting::benchmarking::Interface for Runtime {
    fn account_id() -> <Self as frame_system::Config>::AccountId {
        AccountId::from(ALICE)
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
