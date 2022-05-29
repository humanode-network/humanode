//! Benchmark for pallet-bioauth extrinsics.

use frame_benchmarking::benchmarks;
use frame_support::traits::{Get, Hooks};
use frame_system::RawOrigin;
use crate::Pallet as Bioauth;

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
    }

    authenticate {
        let i in 0..T::MaxAuthentications::get();

        let pubkey = make_pubkey(i);
        let nonce = make_nonce("nonce", i);
        let auth_ticket = T::build(pubkey, nonce);
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

    on_initialize {
        // Populate with expired authentications
        // Setting to 10 for now, can be increased later.
        let mut expired_auths = vec![];
        for i in 0..10 {
            let expired_auth = Authentication {
                public_key: make_pubkey(i),
                expires_at: 1
            };
            expired_auths.push(expired_auth);
        }
        // TODO: Save these auths in the state

    }: {
        let current_block_num: u32 = 10;
        Bioauth::<T>::on_initialize(current_block_num.into());
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::benchmarking::new_benchmark_ext(), crate::mock::benchmarking::Benchmark);
}
