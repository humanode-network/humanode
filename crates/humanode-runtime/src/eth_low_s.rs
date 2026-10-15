//! EIP-2 low-`s` signature check for Ethereum transactions.
//!
//! An ECDSA signature `(r, s)` stays valid for the same message and key as `(r, n - s)` with
//! the recovery id flipped, so any transaction can be re-signed with a different hash by anyone.
//! EIP-2 removes this malleability by only accepting signatures with `s <= secp256k1n / 2`.

use pallet_ethereum::Transaction;
use sp_core::H256;

/// Half of the secp256k1 curve order; the largest `s` value EIP-2 accepts.
pub const SECP256K1N_HALF: H256 = H256([
    0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x5d, 0x57, 0x6e, 0x73, 0x57, 0xa4, 0x50, 0x1d, 0xdf, 0xe9, 0x2f, 0x46, 0x68, 0x1b, 0x20, 0xa0,
]);

/// Checks whether the transaction signature `s` value does not exceed `secp256k1n / 2`.
pub fn is_low_s(transaction: &Transaction) -> bool {
    let s = match transaction {
        Transaction::Legacy(t) => t.signature.s(),
        Transaction::EIP2930(t) => &t.s,
        Transaction::EIP1559(t) => &t.s,
    };
    s <= &SECP256K1N_HALF
}
