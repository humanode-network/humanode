use crate::{Pallet as Bioauth, *};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{traits::Get, WeakBoundedVec};
use frame_system::RawOrigin;
use primitives_auth_ticket::{AuthTicket, OpaqueAuthTicket};

fn make_pubkey(idx: u8) -> Vec<u8> {
    let mut pkey = vec![0; 32];
    let mut prefix: Vec<u8> = Vec::from("public_key");
    prefix.push(idx); // Make it "public_key{}"
    pkey[..prefix.len()].copy_from_slice(&prefix);
    pkey
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
    where_clause {
        where T::OpaqueAuthTicket: From<OpaqueAuthTicket>, T::RobonodeSignature: From<Vec<u8>>
    }

    authenticate {
        let i in 0..T::MaxAuthentications::get();
        let public_key = make_pubkey(i as u8);
        let new_ticket = OpaqueAuthTicket::from(
            &AuthTicket {
                public_key,
                authentication_nonce: Vec::from("nonce"),
            }
        );
        let ticket: T::OpaqueAuthTicket = new_ticket.into();
        //let ticket_signature: T::RobonodeSignature = Vec::from("signature").into();
        let authenticate_req = Authenticate {
            ticket,
            ticket_signature: Vec::from("signature").into(),
        };
    }: _(RawOrigin::None, authenticate_req)

    verify {
        // Verify consumed auth_ticket nonces
        let consumed_nonces = ConsumedAuthTicketNonces::<T>::get();
        assert_eq!(consumed_nonces.len(), 1);
        let expected_nonce: WeakBoundedVec<u8, AuthTicketNonceMaxBytes> = Vec::from("nonce").try_into().unwrap();
        assert_eq!(assert_authticket_nonces_are_eq(&expected_nonce, &consumed_nonces[0]), true);


        // Verify active authentications
        // Skip the first authentication which belongs to Alice's dev account.
        // d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
        let active_authentications = ActiveAuthentications::<T>::get();
        assert_eq!(active_authentications.len(), 2);

        // Expected pubkey
        let expected_pubkey = make_pubkey(0 as u8);
        let observed_pubkey: Vec<u8> = active_authentications[1].public_key.encode();
        assert_eq!(observed_pubkey, expected_pubkey);

    }

    impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::Test);
}
