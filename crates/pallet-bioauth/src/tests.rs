use crate as pallet_bioauth;
use crate::*;
use crate::{mock::*, Error};
use frame_support::pallet_prelude::*;
use frame_support::{assert_noop, assert_ok};

pub fn make_input(
    public_key: &[u8],
    nonce: &[u8],
    signature: &[u8],
) -> crate::Authenticate<MockOpaqueAuthTicket, Vec<u8>> {
    crate::Authenticate {
        ticket: MockOpaqueAuthTicket(StoredAuthTicket {
            public_key: public_key.into(),
            nonce: nonce.into(),
        }),
        ticket_signature: signature.into(),
    }
}

#[test]
fn it_permits_authentication_with_an_empty_state() {
    new_test_ext().execute_with(|| {
        // Prepare test input.
        let input = make_input(b"qwe", b"rty", b"should_be_valid");
        let current_block_number = System::block_number();

        assert_ok!(Bioauth::authenticate(Origin::none(), input));
        assert_eq!(
            Bioauth::stored_public_keys(),
            vec![StoredPublicKey {
                public_key: b"qwe".to_vec(),
                expiration_time: current_block_number + LIFE_TIME_CONST,
            }]
        );
        assert_eq!(Bioauth::stored_nonces(), vec![b"rty".to_vec()]);
    });
}

#[test]
fn it_permits_expiration() {
    new_test_ext().execute_with(|| {
        // First authentication by qwe.
        let first_input = make_input(b"qwe", b"first", b"should_be_valid");
        let first_block_number = System::block_number();

        assert_ok!(Bioauth::authenticate(Origin::none(), first_input));

        // Prepare Alice as we need to keep non-empty state for consensus.
        let alice = make_input(b"alice", b"alice", b"should_be_valid");
        let alice_block_number = 10;
        System::set_block_number(alice_block_number);
        assert_ok!(Bioauth::authenticate(Origin::none(), alice));

        assert_eq!(
            Bioauth::stored_public_keys(),
            vec![
                StoredPublicKey {
                    public_key: b"qwe".to_vec(),
                    expiration_time: first_block_number + LIFE_TIME_CONST,
                },
                StoredPublicKey {
                    public_key: b"alice".to_vec(),
                    expiration_time: alice_block_number + LIFE_TIME_CONST,
                }
            ]
        );
        assert_eq!(
            Bioauth::stored_nonces(),
            vec![b"first".to_vec(), b"alice".to_vec()]
        );

        // Initialize first_block_number block to expire first authentication by qwe.
        Bioauth::on_initialize(first_block_number + LIFE_TIME_CONST + 1);

        assert_eq!(
            Bioauth::stored_public_keys(),
            vec![StoredPublicKey {
                public_key: b"alice".to_vec(),
                expiration_time: alice_block_number + LIFE_TIME_CONST,
            }]
        );
        assert_eq!(
            Bioauth::stored_nonces(),
            vec![b"first".to_vec(), b"alice".to_vec()]
        );
    });
}

#[test]
fn it_permits_authentication_when_previous_one_has_been_expired() {
    new_test_ext().execute_with(|| {
        // First authentication by qwe.
        let first_input = make_input(b"qwe", b"first", b"should_be_valid");
        let first_block_number = System::block_number();

        assert_ok!(Bioauth::authenticate(Origin::none(), first_input));

        // Prepare Alice as we need to keep non-empty state for consensus.
        let alice = make_input(b"alice", b"alice", b"should_be_valid");
        let alice_block_number = 10;
        System::set_block_number(alice_block_number);
        assert_ok!(Bioauth::authenticate(Origin::none(), alice));

        assert_eq!(
            Bioauth::stored_public_keys(),
            vec![
                StoredPublicKey {
                    public_key: b"qwe".to_vec(),
                    expiration_time: first_block_number + LIFE_TIME_CONST,
                },
                StoredPublicKey {
                    public_key: b"alice".to_vec(),
                    expiration_time: alice_block_number + LIFE_TIME_CONST,
                }
            ]
        );
        assert_eq!(
            Bioauth::stored_nonces(),
            vec![b"first".to_vec(), b"alice".to_vec()]
        );

        // Initialize first_block_number block to expire first authentication by qwe.
        Bioauth::on_initialize(first_block_number + LIFE_TIME_CONST + 1);

        assert_eq!(
            Bioauth::stored_public_keys(),
            vec![StoredPublicKey {
                public_key: b"alice".to_vec(),
                expiration_time: alice_block_number + LIFE_TIME_CONST,
            }]
        );
        assert_eq!(
            Bioauth::stored_nonces(),
            vec![b"first".to_vec(), b"alice".to_vec()]
        );

        // Second authentication by qwe.
        let second_input = make_input(b"qwe", b"second", b"should_be_valid");
        let second_block_number = 30;
        System::set_block_number(second_block_number);
        assert_ok!(Bioauth::authenticate(Origin::none(), second_input));
        assert_eq!(
            Bioauth::stored_public_keys(),
            vec![
                StoredPublicKey {
                    public_key: b"alice".to_vec(),
                    expiration_time: alice_block_number + LIFE_TIME_CONST,
                },
                StoredPublicKey {
                    public_key: b"qwe".to_vec(),
                    expiration_time: second_block_number + LIFE_TIME_CONST,
                },
            ]
        );
        assert_eq!(
            Bioauth::stored_nonces(),
            vec![b"first".to_vec(), b"alice".to_vec(), b"second".to_vec()]
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
        // First authentication by qwe.
        let first_input = make_input(b"qwe", b"first", b"should_be_valid");
        let first_block_number = System::block_number();

        assert_ok!(Bioauth::authenticate(Origin::none(), first_input));

        // Prepare Alice as we need to keep non-empty state for consensus.
        let alice = make_input(b"alice", b"alice", b"should_be_valid");
        let alice_block_number = 10;
        System::set_block_number(alice_block_number);
        assert_ok!(Bioauth::authenticate(Origin::none(), alice));

        assert_eq!(
            Bioauth::stored_public_keys(),
            vec![
                StoredPublicKey {
                    public_key: b"qwe".to_vec(),
                    expiration_time: first_block_number + LIFE_TIME_CONST,
                },
                StoredPublicKey {
                    public_key: b"alice".to_vec(),
                    expiration_time: alice_block_number + LIFE_TIME_CONST,
                }
            ]
        );
        assert_eq!(
            Bioauth::stored_nonces(),
            vec![b"first".to_vec(), b"alice".to_vec()]
        );

        // Initialize first_block_number block to expire first authentication by qwe.
        Bioauth::on_initialize(first_block_number + LIFE_TIME_CONST + 1);

        assert_eq!(
            Bioauth::stored_public_keys(),
            vec![StoredPublicKey {
                public_key: b"alice".to_vec(),
                expiration_time: alice_block_number + LIFE_TIME_CONST,
            }]
        );
        assert_eq!(
            Bioauth::stored_nonces(),
            vec![b"first".to_vec(), b"alice".to_vec()]
        );

        // Second authentication by qwe.
        let second_input = make_input(b"qwe", b"first", b"should_be_valid");
        let second_block_number = 30;
        System::set_block_number(second_block_number);
        assert_noop!(
            Bioauth::authenticate(Origin::none(), second_input),
            Error::<Test>::NonceAlreadyUsed,
        );
        assert_eq!(
            Bioauth::stored_public_keys(),
            vec![StoredPublicKey {
                public_key: b"alice".to_vec(),
                expiration_time: alice_block_number + LIFE_TIME_CONST,
            }]
        );
        assert_eq!(
            Bioauth::stored_nonces(),
            vec![b"first".to_vec(), b"alice".to_vec()]
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
            Bioauth::stored_public_keys(),
            vec![StoredPublicKey {
                public_key: b"conflict!".to_vec(),
                expiration_time: current_block_number + LIFE_TIME_CONST,
            }]
        );
        assert_eq!(Bioauth::stored_nonces(), vec![b"nonce1".to_vec()]);
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
        let expected_tag = StoredAuthTicket {
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
