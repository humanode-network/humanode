//! Benchmark for pallet-bioauth extrinsics.

use frame_benchmarking::benchmarks;
use frame_support::traits::{Get, Hooks};
use frame_system::RawOrigin;

use crate::Pallet as Bioauth;
use crate::*;

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
pub trait RobonodePublicKeyBuilder: pallet::Config {
    /// Build a value of the `RobonodePublicKey` type for a given variant.
    fn build(value: RobonodePublicKeyBuilderValue) -> <Self as pallet::Config>::RobonodePublicKey;
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

/// Enables construction of AuthTicket deterministically.
pub trait AuthTicketBuilder: pallet::Config {
    /// Make `AuthTicket` with predetermined 32 bytes public key and nonce.
    fn build(public_key: Vec<u8>, nonce: Vec<u8>) -> <Self as pallet::Config>::OpaqueAuthTicket;
}

/// Enables generation of signature with robonode private key provided at runtime.
pub trait AuthTicketSigner: pallet::Config {
    /// Signs `AuthTicket` bytearray provided and returns digitial signature in bytearray.
    fn sign(
        auth_ticket: &<Self as pallet::Config>::OpaqueAuthTicket,
    ) -> <Self as pallet::Config>::RobonodeSignature;
}

fn make_authentications<Pubkey: From<[u8; 32]>, Moment: Copy>(
    count: usize,
    expires_at: Moment,
) -> Vec<Authentication<Pubkey, Moment>> {
    let mut auths: Vec<Authentication<Pubkey, Moment>> = vec![];
    for i in 0..count {
        let public_key: [u8; 32] = make_pubkey(i as u32).try_into().unwrap();
        let auth = Authentication {
            public_key: public_key.into(),
            expires_at,
        };
        auths.push(auth);
    }
    auths
}

benchmarks! {
    where_clause {
        where T: AuthTicketBuilder + AuthTicketSigner,
            T::ValidatorPublicKey: From<[u8; 32]>,
            T::Moment: From<u64>,
            T: RobonodePublicKeyBuilder
    }

    authenticate {
        let i in 0..T::MaxAuthentications::get();

        // Create `Authenticate` request payload.
        let public_key = make_pubkey(i);
        let nonce = make_nonce("nonce", i);
        let ticket = <T as AuthTicketBuilder>::build(public_key, nonce);
        let ticket_signature = <T as AuthTicketSigner>::sign(&ticket);
        let req = Authenticate {
            ticket,
            ticket_signature,
        };

        let active_authentications_before = ActiveAuthentications::<T>::get().len();
        let consumed_nonces_before = ConsumedAuthTicketNonces::<T>::get();

    }: _(RawOrigin::None, req)

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
        // TODO(#374): populate the active authentications set.

        let robonode_public_key_before = RobonodePublicKey::<T>::get();
        let active_authentications_before = ActiveAuthentications::<T>::get();
        let consumed_nonces_before = ConsumedAuthTicketNonces::<T>::get();

        let new_robonode_public_key = <T as RobonodePublicKeyBuilder>::build(RobonodePublicKeyBuilderValue::B);

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

    on_initialize {
        let b in 1..100;
        let block_num = b;
        let expired_auth_count = 100;
        let active_auth_count = 10;

        let mut auths: Vec<Authentication<T::ValidatorPublicKey, T::Moment>> = vec![];
        // Populate with expired authentications.
        let mut expired_auths = make_authentications(expired_auth_count, T::CurrentMoment::now());
        auths.append(&mut expired_auths);

        // Also, populate with active authentications.
        let future_expiry = T::CurrentMoment::now() + (10u64).into();
        let mut active_auths = make_authentications(active_auth_count, future_expiry);
        auths.append(&mut active_auths);

        let weakly_bound_auths = WeakBoundedVec::force_from(auths, Some("pallet-bioauth:benchmark:on_initialize"));
        ActiveAuthentications::<T>::put(weakly_bound_auths);

        // Capture this state for comparison.
        let auths_before = ActiveAuthentications::<T>::get();
    }: {
        Bioauth::<T>::on_initialize(block_num.into());
    }

    verify {
        let auths_after = ActiveAuthentications::<T>::get();
        assert_eq!(auths_before.len() - auths_after.len(), expired_auth_count);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::benchmarking::new_benchmark_ext(), crate::mock::benchmarking::Benchmark);
}
