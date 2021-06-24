use crate as pallet_bioauth;
use crate::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, weights::DispatchInfo};

pub fn make_input(public_key: &[u8], nonce: &[u8], signature: &[u8]) -> crate::Authenticate {
    let ticket =
        primitives_auth_ticket::OpaqueAuthTicket::from(&primitives_auth_ticket::AuthTicket {
            public_key: Vec::from(public_key),
            authentication_nonce: Vec::from(nonce),
        });
    crate::Authenticate {
        ticket: ticket.into(),
        ticket_signature: Vec::from(signature),
    }
}

#[test]
fn it_permits_authnetication_with_an_empty_state() {
    new_test_ext().execute_with(|| {
        // Prepare test input.
        let input = make_input(b"qwe", b"rty", b"should_be_valid");

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
fn it_denies_authnetication_with_invalid_signature() {
    new_test_ext().execute_with(|| {
        // Prepare test input.
        let input = make_input(b"qwe", b"rty", b"invalid");

        assert_noop!(
            Bioauth::authenticate(Origin::signed(1), input),
            Error::<Test>::AuthTicketSignatureInvalid
        );
    });
}

#[test]
fn it_denies_authnetication_with_conlicting_nonce() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let precondition_input = make_input(b"pk1", b"conflict!", b"should_be_valid");
        assert_ok!(Bioauth::authenticate(Origin::signed(1), precondition_input));

        // Prepare test input.
        let input = make_input(b"pk2", b"conflict!", b"should_be_valid");

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
        let precondition_input = make_input(b"conflict!", b"nonce1", b"should_be_valid");
        assert_ok!(Bioauth::authenticate(Origin::signed(1), precondition_input));

        // Prepare test input.
        let input = make_input(b"conflict!", b"nonce2", b"should_be_valid");

        // Ensure the expected error is thrown when no value is present.
        assert_noop!(
            Bioauth::authenticate(Origin::signed(1), input),
            Error::<Test>::PublicKeyAlreadyUsed,
        );
    });
}

#[test]
fn signed_ext_check_bioauth_tx_deny_invalid_signature() {
    new_test_ext().execute_with(|| {
        // Prepare test input.
        let input = make_input(b"qwe", b"rty", b"invalid");

        let call = <pallet_bioauth::Call<Test>>::authenticate(input).into();
        let info = DispatchInfo::default();

        assert_eq!(
            CheckBioauthTx::<Test>(PhantomData).validate(&1, &call, &info, 1),
            InvalidTransaction::Call.into()
        );
    })
}

#[test]
fn signed_ext_check_bioauth_tx_permit_empty_state() {
    new_test_ext().execute_with(|| {
        // Prepare test input.
        let input = make_input(b"qwe", b"rty", b"should_be_valid");

        let call = <pallet_bioauth::Call<Test>>::authenticate(input).into();
        let info = DispatchInfo::default();

        assert_eq!(
            CheckBioauthTx::<Test>(PhantomData).validate(&1, &call, &info, 1),
            Ok(ValidTransaction::default())
        );
    })
}

#[test]
fn signed_ext_check_bioauth_tx_deny_conlicting_nonce() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let precondition_input = make_input(b"pk1", b"conflict!", b"should_be_valid");
        assert_ok!(Bioauth::authenticate(Origin::signed(1), precondition_input));

        // Prepare test input.
        let input = make_input(b"pk2", b"conflict!", b"should_be_valid");

        let call = <pallet_bioauth::Call<Test>>::authenticate(input).into();
        let info = DispatchInfo::default();

        assert_eq!(
            CheckBioauthTx::<Test>(PhantomData).validate(&1, &call, &info, 1),
            InvalidTransaction::Call.into()
        );
    })
}

#[test]
fn signed_ext_check_bioauth_tx_deny_public_keys() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let precondition_input = make_input(b"conflict!", b"nonce1", b"should_be_valid");
        assert_ok!(Bioauth::authenticate(Origin::signed(1), precondition_input));

        // Prepare test input.
        let input = make_input(b"conflict!", b"nonce2", b"should_be_valid");

        let call = <pallet_bioauth::Call<Test>>::authenticate(input).into();
        let info = DispatchInfo::default();

        assert_eq!(
            CheckBioauthTx::<Test>(PhantomData).validate(&1, &call, &info, 1),
            InvalidTransaction::Call.into()
        );
    })
}
