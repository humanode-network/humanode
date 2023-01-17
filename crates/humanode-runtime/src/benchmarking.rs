//! The benchmarking utilities.

// Allow simple integer arithmetic in tests.
#![allow(clippy::integer_arithmetic)]

use eip712_common::keccak_256;
use frame_support::{
    dispatch::DispatchResult,
    traits::{OnFinalize, OnInitialize},
};
use primitives_ethereum::{EcdsaSignature, EthereumAddress};
use sp_runtime::traits::{One, Zero};

use super::*;

const START_TIMESTAMP: UnixMilliseconds = 1000;
const VESTING_BALANCE: u128 = 1000;
const CLIFF: UnixMilliseconds = 1000;
const VESTING_DURATION: UnixMilliseconds = 3000;

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

fn switch_block<
    T: frame_system::Config + pallet_timestamp::Config + pallet_chain_start_moment::Config,
>() {
    if frame_system::Pallet::<T>::block_number() != Zero::zero() {
        pallet_timestamp::Pallet::<T>::on_finalize(frame_system::Pallet::<T>::block_number());
        pallet_chain_start_moment::Pallet::<T>::on_finalize(
            frame_system::Pallet::<T>::block_number(),
        );
        frame_system::Pallet::<T>::on_finalize(frame_system::Pallet::<T>::block_number());
    }
    frame_system::Pallet::<T>::set_block_number(
        frame_system::Pallet::<T>::block_number() + One::one(),
    );
    frame_system::Pallet::<T>::on_initialize(frame_system::Pallet::<T>::block_number());
    pallet_timestamp::Pallet::<T>::on_initialize(frame_system::Pallet::<T>::block_number());
    pallet_chain_start_moment::Pallet::<T>::on_initialize(frame_system::Pallet::<T>::block_number());
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

        let chain_id: [u8; 32] = U256::from(crate::eth_sig::ETHEREUM_MAINNET_CHAIN_ID).into();
        let verifying_contract = crate::eth_sig::genesis_verifying_contract();
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

    fn prepare() -> Self::Data {
        // Run blocks to be vesting schedule ready.
        switch_block::<Runtime>();
        pallet_timestamp::Pallet::<Runtime>::set(RuntimeOrigin::none(), START_TIMESTAMP).unwrap();
        switch_block::<Runtime>();
    }

    fn verify(_data: Self::Data) -> DispatchResult {
        Ok(())
    }
}

impl pallet_vesting::benchmarking::Interface for Runtime {
    fn account_id() -> <Self as frame_system::Config>::AccountId {
        AccountId::from(ALICE)
    }

    fn schedule() -> <Self as pallet_vesting::Config>::Schedule {
        use vesting_schedule_linear::LinearSchedule;
        vec![LinearSchedule {
            balance: VESTING_BALANCE,
            cliff: CLIFF,
            vesting: VESTING_DURATION,
        }]
        .try_into()
        .unwrap()
    }
}

impl pallet_vesting::benchmarking::SchedulingDriver for vesting::SchedulingDriver {
    type Data = ();

    fn prepare_init() -> Self::Data {
        // Run blocks to be vesting schedule ready.
        switch_block::<Runtime>();
        pallet_timestamp::Pallet::<Runtime>::set(RuntimeOrigin::none(), START_TIMESTAMP).unwrap();
        switch_block::<Runtime>();
    }

    fn prepare_advance(_data: Self::Data) -> Self::Data {
        // Run blocks with setting proper timestamp to make full unlocking.
        pallet_timestamp::Pallet::<Runtime>::set(
            RuntimeOrigin::none(),
            START_TIMESTAMP + CLIFF + VESTING_DURATION,
        )
        .unwrap();
        switch_block::<Runtime>();
    }

    fn verify(_data: Self::Data) -> DispatchResult {
        Ok(())
    }
}
