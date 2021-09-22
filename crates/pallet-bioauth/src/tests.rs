use crate as pallet_bioauth;
use crate::*;
use crate::{mock::*, Error};
use frame_support::pallet_prelude::*;
use frame_support::{assert_noop, assert_ok};
use mockall::predicate;

pub fn make_input(
    public_key: &[u8],
    nonce: &[u8],
    signature: &[u8],
) -> crate::Authenticate<MockOpaqueAuthTicket, Vec<u8>> {
    crate::Authenticate {
        ticket: MockOpaqueAuthTicket(AuthTicket {
            public_key: public_key.into(),
            nonce: nonce.into(),
        }),
        ticket_signature: signature.into(),
    }
}

/// This test verifies that authentication call works correctly when the state of the chain is
/// empty.
#[test]
fn authentication_with_empty_state() {
    new_test_ext().execute_with(|| {
        // Prepare test input.
        let input = make_input(b"qwe", b"rty", b"should_be_valid");
        let current_block_number = System::block_number();

        // Set up mock expectations.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set()
                .with(predicate::eq(vec![b"qwe".to_vec()]))
                .return_const(());
        });

        assert_ok!(Bioauth::authenticate(Origin::none(), input));
        assert_eq!(
            Bioauth::active_authentications(),
            vec![Authentication {
                public_key: b"qwe".to_vec(),
                expires_at: current_block_number + AUTHENTICATIONS_EXPIRE_AFTER_BLOCKS,
            }]
        );
        assert_eq!(
            Bioauth::consumed_auth_ticket_nonces(),
            vec![b"rty".to_vec()]
        );

        with_mock_validator_set_updater(|mock| mock.checkpoint());
    });
}

#[test]
fn authentication_expires() {
    new_test_ext().execute_with(|| {
        // Prepare the test preconditions.
        let current_block_number = System::block_number();
        let expires_at = current_block_number + 1;
        <ActiveAuthentications<Test>>::put(vec![Authentication {
            public_key: b"alice_pk".to_vec(),
            expires_at,
        }]);
        <ConsumedAuthTicketNonces<Test>>::put(vec![b"alice_auth_ticket_nonce".to_vec()]);

        // Set up mock expectations.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set()
                .with(predicate::eq(vec![]))
                .return_const(());
        });

        Bioauth::on_initialize(expires_at);

        // Ensure that authentication expires.
        assert_eq!(Bioauth::active_authentications(), vec![]);
        // Ensure that nonce didn't go anywhere as is still listed as blocked.
        assert_eq!(
            Bioauth::consumed_auth_ticket_nonces(),
            vec![b"alice_auth_ticket_nonce".to_vec()]
        );
    });
}

#[test]
fn it_permits_authentication_when_previous_one_has_been_expired() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let alice_expiration = System::block_number() + AUTHENTICATIONS_EXPIRE_AFTER_BLOCKS + 100;
        let bob_expiration = System::block_number() + AUTHENTICATIONS_EXPIRE_AFTER_BLOCKS;
        <ActiveAuthentications<Test>>::put(vec![
            Authentication {
                public_key: b"alice".to_vec(),
                expires_at: alice_expiration,
            },
            Authentication {
                public_key: b"bob".to_vec(),
                expires_at: bob_expiration,
            },
        ]);
        <ConsumedAuthTicketNonces<Test>>::put(vec![b"alice".to_vec(), b"bob".to_vec()]);

        // Prepare the test input.
        let input = make_input(b"bob", b"bob_new", b"should_be_valid");

        // Make test.
        let current_block_number = bob_expiration + 1;
        System::set_block_number(current_block_number);
        Bioauth::on_initialize(current_block_number);

        assert_ok!(Bioauth::authenticate(Origin::none(), input));
        assert_eq!(
            Bioauth::active_authentications(),
            vec![
                Authentication {
                    public_key: b"alice".to_vec(),
                    expires_at: alice_expiration,
                },
                Authentication {
                    public_key: b"bob".to_vec(),
                    expires_at: current_block_number + AUTHENTICATIONS_EXPIRE_AFTER_BLOCKS,
                }
            ]
        );
        assert_eq!(
            Bioauth::consumed_auth_ticket_nonces(),
            vec![b"alice".to_vec(), b"bob".to_vec(), b"bob_new".to_vec()]
        );
    });
}

#[test]
fn it_denies_authentication_with_invalid_signature() {
    new_test_ext().execute_with(|| {
        // Prepare test input.
        let input = make_input(b"qwe", b"rty", b"invalid");

        assert_noop!(
            Bioauth::authenticate(Origin::none(), input),
            Error::<Test>::AuthTicketSignatureInvalid
        );
    });
}

#[test]
fn it_denies_authentication_with_conlicting_nonce() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let precondition_input = make_input(b"pk1", b"conflict!", b"should_be_valid");
        assert_ok!(Bioauth::authenticate(Origin::none(), precondition_input));

        // Prepare test input.
        let input = make_input(b"pk2", b"conflict!", b"should_be_valid");

        // Ensure the expected error is thrown when no value is present.
        assert_noop!(
            Bioauth::authenticate(Origin::none(), input),
            Error::<Test>::NonceAlreadyUsed,
        );
    });
}

#[test]
fn it_denies_authentication_with_conlicting_nonce_after_expiration() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let alice_expiration = System::block_number() + AUTHENTICATIONS_EXPIRE_AFTER_BLOCKS + 100;
        let bob_expiration = System::block_number() + AUTHENTICATIONS_EXPIRE_AFTER_BLOCKS;
        <ActiveAuthentications<Test>>::put(vec![
            Authentication {
                public_key: b"alice".to_vec(),
                expires_at: alice_expiration,
            },
            Authentication {
                public_key: b"bob".to_vec(),
                expires_at: bob_expiration,
            },
        ]);
        <ConsumedAuthTicketNonces<Test>>::put(vec![b"alice".to_vec(), b"bob".to_vec()]);

        // Prepare the test input.
        let input = make_input(b"bob", b"bob", b"should_be_valid");

        // Make test.
        let current_block_number = bob_expiration + 1;
        System::set_block_number(current_block_number);
        Bioauth::on_initialize(current_block_number);

        assert_noop!(
            Bioauth::authenticate(Origin::none(), input),
            Error::<Test>::NonceAlreadyUsed,
        );
        assert_eq!(
            Bioauth::active_authentications(),
            vec![Authentication {
                public_key: b"alice".to_vec(),
                expires_at: alice_expiration,
            }]
        );
        assert_eq!(
            Bioauth::consumed_auth_ticket_nonces(),
            vec![b"alice".to_vec(), b"bob".to_vec()]
        );
    });
}

#[test]
fn it_denies_authentication_with_concurrent_conlicting_public_keys() {
    new_test_ext().execute_with(|| {
        let current_block_number = System::block_number();

        // Prepare the test precondition.
        let precondition_input = make_input(b"conflict!", b"nonce1", b"should_be_valid");
        assert_ok!(Bioauth::authenticate(Origin::none(), precondition_input));

        // Prepare test input.
        let input = make_input(b"conflict!", b"nonce2", b"should_be_valid");

        // Ensure the expected error is thrown when no value is present.
        assert_noop!(
            Bioauth::authenticate(Origin::none(), input),
            Error::<Test>::PublicKeyAlreadyUsed,
        );
        assert_eq!(
            Bioauth::active_authentications(),
            vec![Authentication {
                public_key: b"conflict!".to_vec(),
                expires_at: current_block_number + AUTHENTICATIONS_EXPIRE_AFTER_BLOCKS,
            }]
        );
        assert_eq!(
            Bioauth::consumed_auth_ticket_nonces(),
            vec![b"nonce1".to_vec()]
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
            InvalidTransaction::Custom(b's').into()
        );
    })
}

#[test]
fn signed_ext_check_bioauth_tx_permit_empty_state() {
    new_test_ext().execute_with(|| {
        // Prepare test input.
        let input = make_input(b"qwe", b"rty", b"should_be_valid");
        let expected_tag = AuthTicket {
            public_key: b"qwe".to_vec(),
            nonce: b"rty".to_vec(),
        };

        let call = <pallet_bioauth::Call<Test>>::authenticate(input).into();
        let info = DispatchInfo::default();

        assert_eq!(
            CheckBioauthTx::<Test>(PhantomData).validate(&1, &call, &info, 1),
            ValidTransaction::with_tag_prefix("bioauth")
                .and_provides(expected_tag)
                .priority(50)
                .longevity(1)
                .propagate(true)
                .build()
        );
    })
}

#[test]
fn signed_ext_check_bioauth_tx_deny_conlicting_nonce() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let precondition_input = make_input(b"pk1", b"conflict!", b"should_be_valid");
        assert_ok!(Bioauth::authenticate(Origin::none(), precondition_input));

        // Prepare test input.
        let input = make_input(b"pk2", b"conflict!", b"should_be_valid");

        let call = <pallet_bioauth::Call<Test>>::authenticate(input).into();
        let info = DispatchInfo::default();

        assert_eq!(
            CheckBioauthTx::<Test>(PhantomData).validate(&1, &call, &info, 1),
            InvalidTransaction::Custom(b'c').into()
        );
    })
}

#[test]
fn signed_ext_check_bioauth_tx_deny_public_keys() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let precondition_input = make_input(b"conflict!", b"nonce1", b"should_be_valid");
        assert_ok!(Bioauth::authenticate(Origin::none(), precondition_input));

        // Prepare test input.
        let input = make_input(b"conflict!", b"nonce2", b"should_be_valid");

        let call = <pallet_bioauth::Call<Test>>::authenticate(input).into();
        let info = DispatchInfo::default();

        assert_eq!(
            CheckBioauthTx::<Test>(PhantomData).validate(&1, &call, &info, 1),
            InvalidTransaction::Custom(b'c').into()
        );
    })
}
