use crate::*;
use frame_benchmarking::benchmarks;
use frame_support::traits::Get;
use frame_system::RawOrigin;
use primitives_auth_ticket::{AuthTicket, OpaqueAuthTicket};

fn bounded<const SIZE: usize>(prefix: &'static str, idx: u32) -> [u8; SIZE] {
    let mut s = Vec::from(prefix);
    s.extend_from_slice(&idx.to_ne_bytes()[..]);

    let mut zeros = [0; SIZE];
    zeros[..s.len()].copy_from_slice(&s);
    zeros
}

fn make_pubkey(idx: u32) -> Vec<u8> {
    bounded::<32>("public_key", idx).to_vec()
}

fn make_nonce(idx: u32) -> Vec<u8> {
    bounded::<32>("nonce", idx).to_vec()
}

fn make_signature(idx: u32) -> Vec<u8> {
    bounded::<64>("signature", idx).to_vec()
}

fn make_auth_ticket(public_key: Vec<u8>, authentication_nonce: Vec<u8>) -> OpaqueAuthTicket {
    OpaqueAuthTicket::from(&AuthTicket {
        public_key,
        authentication_nonce,
    })
}

benchmarks! {
    where_clause {
        where
            T::OpaqueAuthTicket: From<OpaqueAuthTicket>,
            T::RobonodeSignature: From<Vec<u8>>
    }

    authenticate {
        let i in 0..T::MaxAuthentications::get();

        let pkey = make_pubkey(i);
        let nonce = make_nonce(i);
        let auth_ticket = make_auth_ticket(pkey.clone(), nonce.clone());
        let auth_ticket_final: T::OpaqueAuthTicket = auth_ticket.into();

        let ticket_signature: T::RobonodeSignature = make_signature(i).into();

        let authenticate_req = Authenticate {
            ticket: auth_ticket_final,
            ticket_signature,
        };

        // Skip the first authentication which belongs to Alice's dev account.
        // d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
        let active_authentications_before = ActiveAuthentications::<T>::get();

    }: _(RawOrigin::None, authenticate_req)

    verify {
        // Verify consumed auth_ticket nonces
        let consumed_nonces = ConsumedAuthTicketNonces::<T>::get();
        assert_eq!(consumed_nonces.len(), 1);
        assert!(nonce == consumed_nonces[0].to_vec());

        // Verify active authentications
        let active_authentications_after = ActiveAuthentications::<T>::get();
        assert_eq!(active_authentications_after.len() - active_authentications_before.len(), 1);

        // Expected pubkey
        let observed_pubkey: Vec<u8> = active_authentications_after[active_authentications_after.len() - 1].public_key.encode();
        assert_eq!(pkey, observed_pubkey);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
