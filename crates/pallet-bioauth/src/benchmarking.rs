use crate::*;
use frame_benchmarking::benchmarks;
use frame_support::{traits::Get, WeakBoundedVec};
use frame_system::RawOrigin;
use primitives_auth_ticket::{AuthTicket, OpaqueAuthTicket};

fn make_pubkey(idx: u8) -> Vec<u8> {
    let mut s = Vec::from("public_key");
    s.push(idx);
    bounded(s, 32)
}

fn make_auth_ticket(public_key: Vec<u8>) -> OpaqueAuthTicket {
    let authentication_nonce = Vec::from("nonce");
    OpaqueAuthTicket::from(&AuthTicket {
        public_key,
        authentication_nonce,
    })
}

fn make_signature(idx: u8) -> Vec<u8> {
    let mut s = Vec::from("signature");
    s.push(idx);
    bounded(s, 64)
}

fn bounded(prefix: Vec<u8>, size: usize) -> Vec<u8> {
    let mut zeros = vec![0; size];
    zeros[..prefix.len()].copy_from_slice(&prefix);
    zeros
}

benchmarks! {
    where_clause {
        where T::OpaqueAuthTicket: From<OpaqueAuthTicket>, T::RobonodeSignature: From<Vec<u8>>
    }

    authenticate {
        let i in 0..T::MaxAuthentications::get();

        let pkey = make_pubkey(i as u8);
        let auth_ticket = make_auth_ticket(pkey.to_vec().clone());
        let opaque_auth_ticket: T::OpaqueAuthTicket = auth_ticket.into();

        let ticket_signature: T::RobonodeSignature = make_signature(i as u8).into();

        let authenticate_req = Authenticate {
            ticket: opaque_auth_ticket,
            ticket_signature,
        };

        let active_authentications_before = ActiveAuthentications::<T>::get().len();

    }: _(RawOrigin::None, authenticate_req)

    verify {
        // Verify consumed auth_ticket nonces
        let consumed_nonces = ConsumedAuthTicketNonces::<T>::get();
        assert_eq!(consumed_nonces.len(), 1);
        let expected_nonce: WeakBoundedVec<u8, AuthTicketNonceMaxBytes> = Vec::from("nonce").try_into().unwrap();
        assert!(&expected_nonce == &consumed_nonces[0]);

        // According the fact that benhcmarking can be used for different chain specifications
        // we just need to properly compare the size of active authentications before and after running benchmarks.
        let active_authentications_after = ActiveAuthentications::<T>::get();
        assert_eq!(active_authentications_after.len() - active_authentications_before, 1);

        // Expected pubkey
        let expected_pubkey = make_pubkey(0 as u8);
        let observed_pubkey: Vec<u8> = active_authentications_after[0].public_key.encode();
        assert_eq!(observed_pubkey, expected_pubkey);

    }

    impl_benchmark_test_suite!(Pallet, crate::mock::benchmarking::new_benchmark_ext(), crate::mock::benchmarking::Benchmark);
}
