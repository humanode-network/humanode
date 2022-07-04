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

/// Generate 32 bytes pubkey with
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

/// Convenient function to generate an [`Authentication`] struct in bulk with custom prefix
/// for public key.
fn make_authentications<Pubkey: From<[u8; 32]>, Moment: Copy>(
    prefix: &str,
    count: usize,
    expires_at: Moment,
) -> Vec<Authentication<Pubkey, Moment>> {
    (0..count)
        .into_iter()
        .map(|i| {
            let public_key: [u8; 32] = make_pubkey(prefix, i as u32).try_into().unwrap();
            Authentication {
                public_key: public_key.into(),
                expires_at,
            }
        })
        .collect()
}

/// Populate storage with nonces to emulate blockchain condition under load.
fn populate_nonces<Runtime: pallet::Config>(count: u32) {
    let consumed_nonces: Vec<_> = (0..count)
        .into_iter()
        .map(|n| {
            let nonce = make_nonce("consumed_nonce", n);
            BoundedAuthTicketNonce::try_from(nonce).unwrap()
        })
        .collect();
    let weakly_bounded_consumed_nonces =
        WeakBoundedVec::<_, Runtime::MaxNonces>::try_from(consumed_nonces).unwrap();
    ConsumedAuthTicketNonces::<Runtime>::put(weakly_bounded_consumed_nonces);
}

/// Populate `ActiveAuthentications` storage with active authentications with custom prefix in public key.
fn populate_active_auths<Runtime: pallet::Config>(count: u32)
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

    let weakly_bounded_active_auths =
        WeakBoundedVec::<_, Runtime::MaxAuthentications>::try_from(active_auths).unwrap();
    ActiveAuthentications::<Runtime>::put(weakly_bounded_active_auths);
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
        // Populate authentications and nonces close to maximum capacity.
        populate_active_auths::<T>(T::MaxAuthentications::get() - 1);
        populate_nonces::<T>(T::MaxNonces::get() - 1);

        // Create `authenticate` extrinsic payload.
        let public_key = make_pubkey("new", T::MaxAuthentications::get());
        let nonce = make_nonce("nonce", T::MaxNonces::get());
        let ticket = <T as AuthTicketBuilder>::build(public_key.clone(), nonce);
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
        let observed_pubkey: Vec<u8> = active_authentications_after[active_authentications_before as usize].public_key.encode();
        assert_eq!(observed_pubkey, public_key);
    }

    set_robonode_public_key {
        let authentications_count = T::MaxAuthentications::get() - 1;
        populate_active_auths::<T>(authentications_count);

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
        let block_num = 100_u32;
        let active_auth_count: u32 = T::MaxAuthentications::get() / 2;
        let expiring_auth_count: u32 = T::MaxAuthentications::get() - active_auth_count - 1;

        let mut auths: Vec<Authentication<T::ValidatorPublicKey, T::Moment>> = vec![];
        // Populate with expired authentications.
        let mut expiring_auths = make_authentications("expired", expiring_auth_count as usize, T::CurrentMoment::now());
        auths.append(&mut expiring_auths);

        // Also, populate with active authentications.
        let future_expiry = T::CurrentMoment::now() + (10u64).into();
        let mut active_auths = make_authentications("active", active_auth_count as usize, future_expiry);
        auths.append(&mut active_auths);

        let weakly_bound_auths = WeakBoundedVec::try_from(auths).unwrap();
        ActiveAuthentications::<T>::put(weakly_bound_auths);

        // Capture this state for comparison.
        let auths_before = ActiveAuthentications::<T>::get();
    }: {
        Bioauth::<T>::on_initialize(block_num.into());
    }

    verify {
        let auths_after = ActiveAuthentications::<T>::get();
        assert_eq!(auths_before.len() - auths_after.len(), expiring_auth_count as usize);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::benchmarking::new_benchmark_ext(), crate::mock::benchmarking::Benchmark);
}
