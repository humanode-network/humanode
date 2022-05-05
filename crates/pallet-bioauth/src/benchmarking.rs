use crate::{Pallet as Bioauth, *};
use codec::alloc::string::ToString;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{traits::Get, WeakBoundedVec};
use frame_system::RawOrigin;
use hex_literal::hex;
use primitives_auth_ticket::{AuthTicket, OpaqueAuthTicket};
use robonode_crypto::{Keypair, Signer};

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

fn sign_auth_ticket(robonode_pubkey: &Keypair, auth_ticket: Vec<u8>) -> Vec<u8> {
    let ticket_signature = robonode_pubkey
        .try_sign(&auth_ticket[1..].as_ref())
        .unwrap();
    ticket_signature.to_bytes().to_vec()
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
        let robonode_keypair = Keypair::from_bytes(
            hex!("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a").as_ref()
        ).unwrap();

        // Generate public key for an authentication
        let pkey = make_pubkey(i as u8);
        let auth_ticket = make_auth_ticket(pkey.to_vec().clone());

        let ticket: T::OpaqueAuthTicket = auth_ticket.into();
        let ticket_signature_bytes_vec = sign_auth_ticket(&robonode_keypair, ticket.encode());
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
        let expected_pubkey = make_pubkey(i as u8);
        let observed_pubkey: Vec<u8> = active_authentications_after[active_authentications_before as usize].public_key.encode();
        assert_eq!(observed_pubkey, expected_pubkey);

    }

    impl_benchmark_test_suite!(Pallet, crate::mock::benchmarking::new_benchmark_ext(), crate::mock::benchmarking::Benchmark);
}
