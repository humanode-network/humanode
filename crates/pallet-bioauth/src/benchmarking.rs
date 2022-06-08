//! Benchmark for pallet-bioauth extrinsics.

use frame_benchmarking::benchmarks;
use frame_support::traits::Get;
use frame_system::RawOrigin;

use crate::*;

/// Enables construction of AuthTicket deterministically.
pub trait AuthTicketBuilder {
    /// Make `AuthTicket` with predetermined 32 bytes public key and nonce.
    fn build(public_key: Vec<u8>, nonce: Vec<u8>) -> Vec<u8>;
}

/// Enables generation of signature with robonode private key provided at runtime.
pub trait AuthTicketSigner {
    /// Signs `AuthTicket` bytearray provided and returns digitial signature in bytearray.
    fn sign(auth_ticket: &[u8]) -> Vec<u8>;
}

/// The robonode public key has to be deterministic, but we don't want to specify the exact key
/// values.
/// We require all benchmarkable runtimes to implement two possible robonode keys.
pub enum RobonodePublicKeyBuilderValue {
    /// Variant A, should also be the default.
    A,
    /// Variant B.
    B,
}

/// Provides the robonode public key to the benchmarks.
pub trait RobonodePublicKeyBuilder<RobonodePublicKey> {
    /// Build a value of the `RobonodePublicKey` type for a given variant.
    fn build(value: RobonodePublicKeyBuilderValue) -> RobonodePublicKey;
}

fn make_pubkey(idx: u32) -> Vec<u8> {
    let idx_in_bytes = idx.to_le_bytes();
    let mut pubkey = vec![0; 32];
    let _ = &pubkey[0..idx_in_bytes.len()].copy_from_slice(&idx_in_bytes);
    pubkey
}

fn make_nonce(prefix: &str, idx: u32) -> Vec<u8> {
    let mut nonce = Vec::from(prefix);
    let idx_in_bytes = idx.to_le_bytes();
    nonce.extend(idx_in_bytes);
    nonce
}

benchmarks! {
    where_clause {
        where T::OpaqueAuthTicket: From<Vec<u8>>,
            T::RobonodeSignature: From<Vec<u8>>,
            T: AuthTicketBuilder + AuthTicketSigner,
            T: RobonodePublicKeyBuilder<T::RobonodePublicKey>,
    }

    authenticate {
        let i in 0..T::MaxAuthentications::get();

        let pubkey = make_pubkey(i);
        let nonce = make_nonce("nonce", i);
        let auth_ticket = <T as AuthTicketBuilder>::build(pubkey, nonce);
        let ticket_signature_bytes_vec = T::sign(auth_ticket.as_ref());
        let ticket: T::OpaqueAuthTicket = auth_ticket.into();
        let ticket_signature: T::RobonodeSignature = ticket_signature_bytes_vec.into();

        let authenticate_req = Authenticate {
            ticket,
            ticket_signature,
        };

        let active_authentications_before = ActiveAuthentications::<T>::get().len();
        let consumed_nonces_before = ConsumedAuthTicketNonces::<T>::get();

    }: _(RawOrigin::None, authenticate_req)
    verify {
        // Verify nonce count
        let consumed_nonces_after = ConsumedAuthTicketNonces::<T>::get();
        assert_eq!(consumed_nonces_after.len() - consumed_nonces_before.len(), 1);

        // Bsed on the fact that benhcmarking can be used for different chain specifications,
        // we just need to properly compare the size of active authentications before and after running benchmarks.
        let active_authentications_after = ActiveAuthentications::<T>::get();
        assert_eq!(active_authentications_after.len() - active_authentications_before, 1);

        // Verify public key
        let expected_pubkey = make_pubkey(i);
        let observed_pubkey: Vec<u8> = active_authentications_after[active_authentications_before as usize].public_key.encode();
        assert_eq!(observed_pubkey, expected_pubkey);
    }

    set_robonode_public_key {
        let robonode_public_key_before = RobonodePublicKey::<T>::get();
        let active_authentications_before = ActiveAuthentications::<T>::get();
        let consumed_nonces_before = ConsumedAuthTicketNonces::<T>::get();

        let new_robonode_public_key = <T as RobonodePublicKeyBuilder<T::RobonodePublicKey>>::build(RobonodePublicKeyBuilderValue::B);

        assert_ne!(robonode_public_key_before, new_robonode_public_key);

    }: _(RawOrigin::Root, new_robonode_public_key.clone())
    verify {
        let robonode_public_key_after = RobonodePublicKey::<T>::get();
        let active_authentications_after = ActiveAuthentications::<T>::get();
        let consumed_nonces_after = ConsumedAuthTicketNonces::<T>::get();

        assert_eq!(robonode_public_key_after, new_robonode_public_key);
        assert!(active_authentications_after == vec![]);
        assert!(consumed_nonces_after == consumed_nonces_before);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::benchmarking::new_benchmark_ext(), crate::mock::benchmarking::Benchmark);
}
