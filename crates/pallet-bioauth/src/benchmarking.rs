//! Benchmark for pallet-bioauth extrinsics.

// Allow integer and float arithmetic in tests.
#![allow(clippy::arithmetic_side_effects, clippy::float_arithmetic)]

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

/// Generate 32 bytes pubkey with prefix of our choice.
/// NOTE: `prefix` and `idx` must be strictly under 32 bytes
fn make_pubkey(prefix: &str, idx: u32) -> Vec<u8> {
    let mut pubkey = vec![0; 32];
    let prefix_in_bytes = Vec::from(prefix);
    let idx_in_bytes = idx.to_le_bytes();
    if prefix_in_bytes.len() + idx_in_bytes.len() > 32 {
        panic!("`prefix` and `idx` exceeds 32 bytes");
    }
    let _ = &pubkey[0..prefix_in_bytes.len()].copy_from_slice(&prefix_in_bytes);
    let _ = &pubkey[prefix_in_bytes.len()..(prefix_in_bytes.len() + idx_in_bytes.len())]
        .copy_from_slice(&idx_in_bytes);
    pubkey
}

/// A helper function to make nonce.
fn make_nonce(prefix: &str, idx: u32) -> Vec<u8> {
    let mut nonce = Vec::from(prefix);
    let idx_in_bytes = idx.to_le_bytes();
    nonce.extend(idx_in_bytes);
    nonce
}

/// Enables construction of [`AuthTicket`]s deterministically.
pub trait AuthTicketBuilder: pallet::Config {
    /// Make `AuthTicket` with predetermined 32 bytes public key and nonce.
    fn build(public_key: Vec<u8>, nonce: Vec<u8>) -> <Self as pallet::Config>::OpaqueAuthTicket;
}

/// Enables generation of signature with robonode private key provided at runtime.
pub trait AuthTicketSigner: pallet::Config {
    /// Signs `AuthTicket` bytearray provided and returns digital signature in bytearray.
    fn sign(
        auth_ticket: &<Self as pallet::Config>::OpaqueAuthTicket,
    ) -> <Self as pallet::Config>::RobonodeSignature;
}

/// Convenient function to generate an [`Authentication`] struct in bulk with custom prefix
/// for public key.
fn make_authentications<Pubkey: From<[u8; 32]>, Moment: Copy>(
    prefix: &str,
    count: usize,
    expires_at: Moment,
) -> Vec<Authentication<Pubkey, Moment>> {
    (0..count)
        .map(|i| {
            let public_key: [u8; 32] = make_pubkey(prefix, i.try_into().unwrap())
                .try_into()
                .unwrap();
            Authentication {
                public_key: public_key.into(),
                expires_at,
            }
        })
        .collect()
}

/// Populate the [`ActiveAuthentications`] storage with generated data.
fn populate_active_authentications<Runtime: pallet::Config>(count: u32)
where
    <Runtime as pallet::Config>::ValidatorPublicKey: From<[u8; 32]>,
    <Runtime as pallet::Config>::Moment: From<u64>,
{
    let expiry = Runtime::CurrentMoment::now() + (10u64).into();
    let active_auths = make_authentications::<Runtime::ValidatorPublicKey, Runtime::Moment>(
        "active",
        count as usize,
        expiry,
    );

    let bounded_active_auths =
        BoundedVec::<_, Runtime::MaxAuthentications>::try_from(active_auths).unwrap();
    ActiveAuthentications::<Runtime>::put(bounded_active_auths);
}

/// Populate the [`ConsumedAuthTicketNonces`] storage with generated data.
fn populate_consumed_auth_ticket_nonces<Runtime: pallet::Config>(count: u32) {
    let mut consumed_nonces: Vec<_> = vec![];
    for i in 0..count {
        let nonce = make_nonce("consumed_nonce", i);
        consumed_nonces.push(BoundedAuthTicketNonce::try_from(nonce).unwrap());
    }
    let bounded_consumed_nonces =
        BoundedVec::<_, Runtime::MaxNonces>::try_from(consumed_nonces).unwrap();
    ConsumedAuthTicketNonces::<Runtime>::put(bounded_consumed_nonces);
}

benchmarks! {
    where_clause {
        where
            T: AuthTicketBuilder + AuthTicketSigner,
            T::ValidatorPublicKey: From<[u8; 32]>,
            T::Moment: From<u64>,
            T: RobonodePublicKeyBuilder
    }

    authenticate {
        // Vary the amount of pre-populated active authentications and consumed nonces.
        // Leave one space spare for the payload to be inserted in this call.
        let a in 0 .. (T::MaxAuthentications::get() - 1) =>  populate_active_authentications::<T>(a);
        let n in 0 .. (T::MaxNonces::get() - 1) => populate_consumed_auth_ticket_nonces::<T>(n);

        // Create `authenticate` extrinsic payload.
        let public_key = make_pubkey("new", T::MaxAuthentications::get());
        let nonce = make_nonce("nonce", T::MaxNonces::get());
        let ticket = <T as AuthTicketBuilder>::build(public_key.clone(), nonce);
        let ticket_signature = <T as AuthTicketSigner>::sign(&ticket);
        let req = Authenticate {
            ticket,
            ticket_signature,
        };

        // Capture some data used during the verification.
        let active_authentications_before_len = ActiveAuthentications::<T>::get().len();
        let consumed_auth_ticket_nonces_before_len = ConsumedAuthTicketNonces::<T>::get().len();

    }: _(RawOrigin::None, req)
    verify {
        // Verify that exactly one active authentication was added.
        let active_authentications_after_len = ActiveAuthentications::<T>::get().len();
        assert_eq!(active_authentications_after_len - active_authentications_before_len, 1);

        // Verify that exactly one consumed auth ticket nonce was added.
        let consumed_auth_ticket_nonces_after_len = ConsumedAuthTicketNonces::<T>::get().len();
        assert_eq!(consumed_auth_ticket_nonces_after_len - consumed_auth_ticket_nonces_before_len, 1);
    }

    set_robonode_public_key {
        // Vary the amount of pre-populated active authentications.
        let a in 0 .. (T::MaxAuthentications::get()) =>  populate_active_authentications::<T>(a);

        // Use constant, yet non-zero amount of nonces, as this call isn't supposed to be dependent
        // anyhow on the nonces.
        populate_consumed_auth_ticket_nonces::<T>(10);

        // Capture this state for comparison.
        let robonode_public_key_before = RobonodePublicKey::<T>::get();
        let active_authentications_before = ActiveAuthentications::<T>::get();
        let consumed_auth_ticket_nonces_before = ConsumedAuthTicketNonces::<T>::get();

        // Prepare the [`set_robonode_public_key`] extrinsic argument.
        let new_robonode_public_key = <T as RobonodePublicKeyBuilder>::build(RobonodePublicKeyBuilderValue::B);

        // Self-check that the new key is different from the old one.
        assert_ne!(robonode_public_key_before, new_robonode_public_key);

    }: _(RawOrigin::Root, new_robonode_public_key.clone())
    verify {
        let robonode_public_key_after = RobonodePublicKey::<T>::get();
        let active_authentications_after = ActiveAuthentications::<T>::get();
        let consumed_auth_ticket_nonces_after = ConsumedAuthTicketNonces::<T>::get();

        assert_eq!(robonode_public_key_after, new_robonode_public_key);
        assert!(active_authentications_after == vec![]);
        assert!(consumed_auth_ticket_nonces_after == consumed_auth_ticket_nonces_before);
    }

    on_initialize {
        let a in 0 .. (T::MaxAuthentications::get());
        let active_auth_count: u32 = a / 2;
        let expiring_auth_count: u32 = a - active_auth_count;

        let mut auths: Vec<Authentication<T::ValidatorPublicKey, T::Moment>> = vec![];
        // Populate with expired authentications.
        let mut expiring_auths = make_authentications("expired", expiring_auth_count as usize, T::CurrentMoment::now());
        auths.append(&mut expiring_auths);

        // Also, populate with active authentications.
        let future_expiry = T::CurrentMoment::now() + (10u64).into();
        let mut active_auths = make_authentications("active", active_auth_count as usize, future_expiry);
        auths.append(&mut active_auths);

        let bounded_auths = BoundedVec::try_from(auths).unwrap();
        ActiveAuthentications::<T>::put(bounded_auths);

        // Capture this state for comparison.
        let active_authentications_before_len = ActiveAuthentications::<T>::get().len();
    }: {
        Bioauth::<T>::on_initialize(100u32.into());
    }
    verify {
        let active_authentications_after_len = ActiveAuthentications::<T>::get().len();
        assert_eq!(active_authentications_before_len - active_authentications_after_len, expiring_auth_count as usize);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::benchmarking::new_benchmark_ext(), crate::mock::benchmarking::Benchmark);
}
