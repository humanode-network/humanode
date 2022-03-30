#![cfg(feature = "runtime-benchmarks")]

use crate::{Pallet as Bioauth, *};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::WeakBoundedVec;
use frame_system::RawOrigin;
use primitives_auth_ticket::AuthTicket;

fn make_auth_ticket(id: u8) -> Vec<u8> {
    // Create public key
    let mut public_key: Vec<u8> = vec![0u8; 32];
    let prefix = vec![112, 107, 101, 121, id]; // "pkey{}"
    &public_key[..prefix.len()].copy_from_slice(&prefix);

    // Create unique authentication_nonce
    let authentication_nonce: Vec<u8> = vec![110, 111, 110, 99, 101, id]; // "nonce{}"

    // Create Auth Ticket
    let auth_ticket = AuthTicket {
        public_key,
        authentication_nonce,
    };
    auth_ticket.encode()
}

fn assert_authticket_nonces_are_eq(
    a: &WeakBoundedVec<u8, AuthTicketNonceMaxBytes>,
    b: &WeakBoundedVec<u8, AuthTicketNonceMaxBytes>,
) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b.iter())
            .fold(true, |prev, (i, j)| prev && i == j)
}

benchmarks! {
    authenticate {
        // TODO: Add loop
        let i = 1;
        let ticket_encoded = make_auth_ticket(i);
        let ticket: T::OpaqueAuthTicket = ticket_encoded.into();
        let ticket_signature: T::RobonodeSignature = Vec::from("signature").into();
        let authenticate_req = Authenticate {
            ticket,
            ticket_signature,
        };
    }: _(RawOrigin::None, authenticate_req)

    verify {
        /// Verify consumed auth_ticket nonces
        let consumed_nonces = ConsumedAuthTicketNonces::<T>::get();
        assert_eq!(consumed_nonces.len(), 1);
        for (i, consumed_nonce) in consumed_nonces.iter().enumerate() {
            let expected_nonce: WeakBoundedVec<u8, AuthTicketNonceMaxBytes> = vec![110, 111, 110, 99, 101, (i + 1) as u8].try_into().unwrap(); /// "nonce{}"
            assert_eq!(assert_authticket_nonces_are_eq(&consumed_nonce, &expected_nonce), true);
        }

        /// Verify active authentications
        /// Skip the first authentication which belongs to Alice's dev account.
        /// d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
        let active_authentications = ActiveAuthentications::<T>::get();
        for (idx, active_auth) in active_authentications.iter().enumerate() {
            if idx == 0 {
                continue
            }
            let mut expected: Vec<u8> = vec![0u8; 32];
            let prefix = vec![112, 107, 101, 121, idx as u8]; // "pkey{}"
            &expected[..prefix.len()].copy_from_slice(&prefix);

            let observed_pubkey: Vec<u8> = active_auth.public_key.encode();
            assert_eq!(observed_pubkey, expected);
        }

    }

    impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::Test);
}
