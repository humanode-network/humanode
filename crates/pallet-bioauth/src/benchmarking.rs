use crate::{Pallet as Bioauth, *};
use codec::alloc::string::ToString;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{traits::Get, WeakBoundedVec};
use frame_system::RawOrigin;

/// Enables construction of AuthTicket with customized public key
pub trait AuthTicketBuilder {
    fn build(idx: u8) -> Vec<u8>;
}

/// Enables generation of signature with robonode private key provided at runtime.
pub trait AuthTicketSigner {
    fn sign(auth_ticket: &[u8]) -> Vec<u8>;
}

benchmarks! {
    where_clause {
        where T::OpaqueAuthTicket: From<Vec<u8>>,
            T::RobonodeSignature: From<Vec<u8>>,
            T: AuthTicketBuilder + AuthTicketSigner,
    }

    authenticate {
        let i in 0..T::MaxAuthentications::get();

        // Generate public key for an authentication
        let auth_ticket = T::build(i as u8);
        let ticket_signature_bytes_vec = T::sign(auth_ticket.as_ref());
        let ticket: T::OpaqueAuthTicket = auth_ticket.into();
        let ticket_signature: T::RobonodeSignature = ticket_signature_bytes_vec.into();

        let authenticate_req = Authenticate {
            ticket,
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
        let mut expected_pubkey = vec![0; 32];
        expected_pubkey[31] = i as u8;
        let observed_pubkey: Vec<u8> = active_authentications_after[active_authentications_before as usize].public_key.encode();
        assert_eq!(observed_pubkey, expected_pubkey);

    }

    impl_benchmark_test_suite!(Pallet, crate::mock::benchmarking::new_benchmark_ext(), crate::mock::benchmarking::Benchmark);
}
