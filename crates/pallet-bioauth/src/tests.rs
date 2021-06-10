use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

pub fn make_input(public_key: &[u8], nonce: &[u8]) -> crate::Authenticate {
    let ticket = primitives_bioauth::OpaqueAuthTicket::from(&primitives_bioauth::AuthTicket {
        public_key: Vec::from(public_key),
        authentication_nonce: Vec::from(nonce),
    });
    crate::Authenticate {
        ticket: ticket.into(),
        ticket_signature: Vec::from(&b"TODO"[..]),
    }
}

#[test]
fn it_permits_authnetication_with_an_empty_state() {
    new_test_ext().execute_with(|| {
        // Prepare test input.
        let input = make_input(b"qwe", b"rty");

        assert_ok!(Bioauth::authenticate(Origin::signed(1), input));
        assert_eq!(
            Bioauth::stored_auth_tickets(),
            Some(vec![crate::StoredAuthTicket {
                public_key: Vec::from(&b"qwe"[..]),
                nonce: Vec::from(&b"rty"[..]),
            }])
        );
    });
}

#[test]
fn it_denies_authnetication_with_conlicting_nonce() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let precondition_input = make_input(b"pk1", b"conflict!");
        assert_ok!(Bioauth::authenticate(Origin::signed(1), precondition_input));

        // Prepare test input.
        let input = make_input(b"pk2", b"conflict!");

        // Ensure the expected error is thrown when no value is present.
        assert_noop!(
            Bioauth::authenticate(Origin::signed(1), input),
            Error::<Test>::NonceAlreadyUsed,
        );
    });
}

#[test]
fn it_denies_authnetication_with_conlicting_public_keys() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let precondition_input = make_input(b"conflict!", b"nonce1");
        assert_ok!(Bioauth::authenticate(Origin::signed(1), precondition_input));

        // Prepare test input.
        let input = make_input(b"conflict!", b"nonce2");

        // Ensure the expected error is thrown when no value is present.
        assert_noop!(
            Bioauth::authenticate(Origin::signed(1), input),
            Error::<Test>::PublicKeyAlreadyUsed,
        );
    });
}
