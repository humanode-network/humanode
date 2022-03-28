#![cfg(feature = "runtime-benchmarks")]

use crate::{Pallet as Bioauth, *};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
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

    // TODO: Add verification code
    verify {


    }

    impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::Test);
}
