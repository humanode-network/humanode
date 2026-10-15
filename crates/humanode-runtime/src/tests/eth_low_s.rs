//! Tests to verify that Ethereum transactions with high-`s` (malleable) signatures are rejected.

// Allow simple integer arithmetic in tests.
#![allow(clippy::arithmetic_side_effects)]

use ethereum::{
    EIP1559Transaction, EIP2930Transaction, LegacyTransaction, LegacyTransactionMessage,
    TransactionAction, TransactionSignature,
};
use fp_self_contained::SelfContainedCall;
use sp_runtime::transaction_validity::InvalidTransaction;

use super::*;
use crate::eth_low_s::{is_low_s, SECP256K1N_HALF};

/// The secp256k1 curve order.
const SECP256K1N: U256 = U256([
    0xbfd25e8cd0364141,
    0xbaaedce6af48a03b,
    0xfffffffffffffffe,
    0xffffffffffffffff,
]);

const CHAIN_ID: u64 = 5234;

/// The secret key the test transactions are signed with.
fn secret() -> libsecp256k1::SecretKey {
    libsecp256k1::SecretKey::parse(&sp_io::hashing::keccak_256(b"Alice")).unwrap()
}

/// The Ethereum address of [`secret`].
fn signer() -> H160 {
    let public = libsecp256k1::PublicKey::from_secret_key(&secret());
    H160::from_slice(&sp_io::hashing::keccak_256(&public.serialize()[1..65])[12..])
}

/// Signs the message hash, returning `(r, s, odd_y_parity)`.
fn sign(msg_hash: H256) -> (H256, H256, bool) {
    let (sig, recovery_id) =
        libsecp256k1::sign(&libsecp256k1::Message::parse(&msg_hash.0), &secret());
    let sig = sig.serialize();
    (
        H256::from_slice(&sig[0..32]),
        H256::from_slice(&sig[32..64]),
        recovery_id.serialize() == 1,
    )
}

/// Produces the malleated counterpart `(n - s, !odd_y_parity)` of a signature.
fn malleate(s: H256, odd_y_parity: bool) -> (H256, bool) {
    let malleated_s = SECP256K1N - U256::from_big_endian(s.as_bytes());
    let mut bytes = [0u8; 32];
    malleated_s.to_big_endian(&mut bytes);
    (H256(bytes), !odd_y_parity)
}

/// Recovers the signer the same way Frontier does, bypassing the low-`s` check.
fn recover(r: H256, s: H256, odd_y_parity: bool, msg_hash: H256) -> H160 {
    let mut sig = [0u8; 65];
    sig[0..32].copy_from_slice(r.as_bytes());
    sig[32..64].copy_from_slice(s.as_bytes());
    sig[64] = u8::from(odd_y_parity);
    let pubkey = sp_io::crypto::secp256k1_ecdsa_recover(&sig, &msg_hash.0)
        .ok()
        .expect("signature must be recoverable");
    H160::from(H256::from(sp_io::hashing::keccak_256(&pubkey)))
}

fn legacy_v(odd_y_parity: bool) -> u64 {
    CHAIN_ID * 2 + 35 + u64::from(odd_y_parity)
}

fn legacy_message() -> LegacyTransactionMessage {
    LegacyTransactionMessage {
        nonce: U256::zero(),
        gas_price: U256::from(1),
        gas_limit: U256::from(21000),
        action: TransactionAction::Call(H160::repeat_byte(0x11)),
        value: U256::from(1),
        input: vec![],
        chain_id: Some(CHAIN_ID),
    }
}

fn legacy_transaction(r: H256, s: H256, odd_y_parity: bool) -> EthereumTransaction {
    let message = legacy_message();
    EthereumTransaction::Legacy(LegacyTransaction {
        nonce: message.nonce,
        gas_price: message.gas_price,
        gas_limit: message.gas_limit,
        action: message.action,
        value: message.value,
        input: message.input,
        signature: TransactionSignature::new(legacy_v(odd_y_parity), r, s).unwrap(),
    })
}

fn eip2930_transaction(r: H256, s: H256, odd_y_parity: bool) -> EthereumTransaction {
    EthereumTransaction::EIP2930(EIP2930Transaction {
        chain_id: CHAIN_ID,
        nonce: U256::zero(),
        gas_price: U256::from(1),
        gas_limit: U256::from(21000),
        action: TransactionAction::Call(H160::repeat_byte(0x11)),
        value: U256::from(1),
        input: vec![],
        access_list: vec![],
        odd_y_parity,
        r,
        s,
    })
}

fn eip1559_transaction(r: H256, s: H256, odd_y_parity: bool) -> EthereumTransaction {
    EthereumTransaction::EIP1559(EIP1559Transaction {
        chain_id: CHAIN_ID,
        nonce: U256::zero(),
        max_priority_fee_per_gas: U256::from(1),
        max_fee_per_gas: U256::from(1),
        gas_limit: U256::from(21000),
        action: TransactionAction::Call(H160::repeat_byte(0x11)),
        value: U256::from(1),
        input: vec![],
        access_list: vec![],
        odd_y_parity,
        r,
        s,
    })
}

/// Computes the signing message hash of a transaction.
fn message_hash(transaction: &EthereumTransaction) -> H256 {
    match transaction {
        EthereumTransaction::Legacy(_) => legacy_message().hash(),
        EthereumTransaction::EIP2930(t) => {
            ethereum::EIP2930TransactionMessage::from(t.clone()).hash()
        }
        EthereumTransaction::EIP1559(t) => {
            ethereum::EIP1559TransactionMessage::from(t.clone()).hash()
        }
    }
}

fn check_self_contained(
    transaction: EthereumTransaction,
) -> Option<Result<H160, TransactionValidityError>> {
    RuntimeCall::Ethereum(pallet_ethereum::Call::transact { transaction }).check_self_contained()
}

/// Signs a transaction built by `build`, then asserts that the original signature is accepted
/// and that its malleated counterpart, while recovering to the same signer, is rejected.
fn assert_rejects_malleated(build: fn(H256, H256, bool) -> EthereumTransaction) {
    sp_io::TestExternalities::default().execute_with(|| {
        // The signature does not affect the message hash; any valid placeholder works.
        let placeholder = H256::from_low_u64_be(1);
        let msg_hash = message_hash(&build(placeholder, placeholder, false));
        let (r, s, odd_y_parity) = sign(msg_hash);
        let (malleated_s, malleated_odd_y_parity) = malleate(s, odd_y_parity);

        // The malleated signature is a valid signature by the same signer.
        assert_eq!(recover(r, s, odd_y_parity, msg_hash), signer());
        assert_eq!(
            recover(r, malleated_s, malleated_odd_y_parity, msg_hash),
            signer()
        );

        let transaction = build(r, s, odd_y_parity);
        assert_eq!(message_hash(&transaction), msg_hash);
        assert!(is_low_s(&transaction));
        assert_eq!(check_self_contained(transaction), Some(Ok(signer())));

        let malleated_transaction = build(r, malleated_s, malleated_odd_y_parity);
        assert_eq!(message_hash(&malleated_transaction), msg_hash);
        assert!(!is_low_s(&malleated_transaction));
        assert_eq!(
            check_self_contained(malleated_transaction),
            Some(Err(TransactionValidityError::Invalid(
                InvalidTransaction::Custom(
                    fp_ethereum::TransactionValidationError::InvalidSignature as u8
                )
            )))
        );
    })
}

#[test]
fn legacy_malleated_signature_rejected() {
    assert_rejects_malleated(legacy_transaction);
}

#[test]
fn eip2930_malleated_signature_rejected() {
    assert_rejects_malleated(eip2930_transaction);
}

#[test]
fn eip1559_malleated_signature_rejected() {
    assert_rejects_malleated(eip1559_transaction);
}

#[test]
fn secp256k1n_half_matches_curve_order() {
    assert_eq!(
        U256::from_big_endian(SECP256K1N_HALF.as_bytes()),
        SECP256K1N / 2
    );
}

#[test]
fn low_s_boundary() {
    let half = U256::from_big_endian(SECP256K1N_HALF.as_bytes());
    let above_half = {
        let mut bytes = [0u8; 32];
        (half + 1).to_big_endian(&mut bytes);
        H256(bytes)
    };
    let r = H256::repeat_byte(0x01);

    for build in [
        legacy_transaction as fn(H256, H256, bool) -> EthereumTransaction,
        eip2930_transaction,
        eip1559_transaction,
    ] {
        assert!(is_low_s(&build(r, SECP256K1N_HALF, false)));
        assert!(!is_low_s(&build(r, above_half, false)));
    }
}
