use std::ops::Div;

use crate::{self as pallet_bioauth, mock::*, *};
use frame_support::{assert_noop, assert_ok, assert_storage_noop, pallet_prelude::*};
use mockall::predicate;

/// An exact value of January 1st 2021 in UNIX time milliseconds.
const JANUARY_1_2021: UnixMilliseconds = 1609459200 * 1000;

/// The time at which the test chain "started".
const CHAIN_START: UnixMilliseconds = JANUARY_1_2021;

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

fn block_to_process_moment(moment: UnixMilliseconds) -> BlockNumber {
    let total_work_time = moment.checked_sub(CHAIN_START).unwrap();
    total_work_time
        .saturating_add(SLOT_DURATION - 1)
        .div(SLOT_DURATION)
}

#[test]
fn test_block_to_process_moment() {
    assert_eq!(block_to_process_moment(CHAIN_START), 0);

    assert_eq!(block_to_process_moment(CHAIN_START + 1), 1);
    assert_eq!(block_to_process_moment(CHAIN_START + SLOT_DURATION), 1);

    assert_eq!(block_to_process_moment(CHAIN_START + SLOT_DURATION + 1), 2);
}

#[test]
#[should_panic]
fn test_block_to_process_moment_before_chain_start() {
    block_to_process_moment(CHAIN_START - 1);
}

/// This test verifies that authentication call works correctly when the state of the chain is
/// empty.
#[test]
fn authentication_with_empty_state() {
    new_test_ext().execute_with(|| {
        // Prepare test input.
        let input = make_input(b"qwe", b"rty", b"should_be_valid");
        let current_moment = CHAIN_START + 2 * SLOT_DURATION;
        let expires_at = current_moment + AUTHENTICATIONS_EXPIRE_AFTER;

        // Set up mock expectations.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set()
                .once()
                .with(predicate::eq(vec![b"qwe".to_vec()]))
                .return_const(());
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().once().with().return_const(current_moment);
        });

        // Ensure that authentication call is processed successfully.
        assert_ok!(Bioauth::authenticate(Origin::none(), input));

        // Ensure that the state of ActiveAuthentications has been updated.
        assert_eq!(
            Bioauth::active_authentications(),
            vec![Authentication {
                public_key: b"qwe".to_vec(),
                expires_at,
            }]
        );
        // Ensure that the state of ConsumedAuthTicketNonces has been updated.
        assert_eq!(
            Bioauth::consumed_auth_ticket_nonces(),
            vec![b"rty".to_vec()]
        );
    });
}

/// This test verifies that authentication expiration logic works correctly after getting
/// the block exactly at the moment the authentication becomes expired.
#[test]
fn authentication_expires_exactly_at_the_moment() {
    new_test_ext().execute_with(|| {
        // Prepare the test preconditions.
        let expires_at = CHAIN_START + 2 * SLOT_DURATION;
        <ActiveAuthentications<Test>>::put(vec![Authentication {
            public_key: b"alice_pk".to_vec(),
            expires_at,
        }]);
        <ConsumedAuthTicketNonces<Test>>::put(vec![b"alice_auth_ticket_nonce".to_vec()]);

        // Set up mock expectations.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set()
                .once()
                .with(predicate::eq(vec![]))
                .return_const(());
        });

        // Set the time to exactly the time at which the authentication expires.
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().once().with().return_const(expires_at);
        });

        // Process the block that certainly has to expire the authentication.
        Bioauth::on_initialize(block_to_process_moment(expires_at));

        // Ensure that authentication expires.
        assert_eq!(Bioauth::active_authentications(), vec![]);
        // Ensure that nonce didn't go anywhere as it's still listed as blocked.
        assert_eq!(
            Bioauth::consumed_auth_ticket_nonces(),
            vec![b"alice_auth_ticket_nonce".to_vec()]
        );
    });
}

/// This test verifies that authentication expiration logic works correctly after getting
/// a block which timestamp is after the moment at which the authentication becomes expired.
#[test]
fn authentication_expires_in_successive_block() {
    new_test_ext().execute_with(|| {
        // Prepare the test preconditions.
        let current_moment = CHAIN_START + 2 * SLOT_DURATION;
        let expires_at = current_moment - 10;
        <ActiveAuthentications<Test>>::put(vec![Authentication {
            public_key: b"alice_pk".to_vec(),
            expires_at,
        }]);
        <ConsumedAuthTicketNonces<Test>>::put(vec![b"alice_auth_ticket_nonce".to_vec()]);

        // Set up mock expectations.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set()
                .once()
                .with(predicate::eq(vec![]))
                .return_const(());
        });

        // Set the time a bit past the moment when the authentication expires.
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().once().with().return_const(current_moment);
        });

        // Process the block that certainly has to expire the authentication.
        Bioauth::on_initialize(block_to_process_moment(expires_at));

        // Ensure that authentication expires.
        assert_eq!(Bioauth::active_authentications(), vec![]);
        // Ensure that nonce didn't go anywhere as it's still listed as blocked.
        assert_eq!(
            Bioauth::consumed_auth_ticket_nonces(),
            vec![b"alice_auth_ticket_nonce".to_vec()]
        );
    });
}

/// This test ensures that authentication remains active for the whole period up until the block
/// with the timestamp past it's expiration time arrives.
#[test]
fn authentication_expiration_lifecycle() {
    new_test_ext().execute_with(|| {
        // Prepare the test preconditions.
        let mut current_moment = CHAIN_START + 2 * SLOT_DURATION;
        let expires_at = current_moment + AUTHENTICATIONS_EXPIRE_AFTER;

        let authnetication = Authentication {
            public_key: b"alice_pk".to_vec(),
            expires_at,
        };
        let nonce = b"alice_auth_ticket_nonce".to_vec();

        <ActiveAuthentications<Test>>::put(vec![authnetication.clone()]);
        <ConsumedAuthTicketNonces<Test>>::put(vec![nonce.clone()]);

        loop {
            // Set up mock expectations.
            with_mock_current_moment_provider(|mock| {
                mock.expect_now().once().with().return_const(current_moment);
            });

            assert_storage_noop!(Bioauth::on_initialize(System::block_number()));

            // Ensure that authentication is still active.
            assert_eq!(
                Bioauth::active_authentications(),
                vec![authnetication.clone()]
            );
            // Ensure that nonce didn't go anywhere and it's still listed as blocked.
            assert_eq!(Bioauth::consumed_auth_ticket_nonces(), vec![nonce.clone()]);

            // Advance the block number and the current moment.
            System::set_block_number(System::block_number() + 1);
            current_moment += SLOT_DURATION;

            // If the current moment is past the expiration moment - we should move to
            // the assertion of the expiration.
            if current_moment >= expires_at {
                break;
            }
        }

        // Only now we expect the code to issue a validators set update.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set()
                .once()
                .with(predicate::eq(vec![]))
                .return_const(());
        });

        // Set the current moment (that has by now adjusted to be past the expiration moment)
        // expectation.
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().once().with().return_const(current_moment);
        });

        // Run the expiration processing on the block that's been adjusted previously.
        Bioauth::on_initialize(System::block_number());

        // Ensure that authentication is gone.
        assert_eq!(Bioauth::active_authentications(), vec![]);
        // Ensure that nonce didn't go anywhere and it's still listed as blocked.
        assert_eq!(Bioauth::consumed_auth_ticket_nonces(), vec![nonce]);
    });
}

/// This test verifies that authentication call works correctly when a previous
/// authentication has been expired.
#[test]
fn authentication_when_previous_one_has_been_expired() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let expires_at = CHAIN_START + 2 * SLOT_DURATION;
        <ActiveAuthentications<Test>>::put(vec![Authentication {
            public_key: b"alice_pk".to_vec(),
            expires_at,
        }]);
        <ConsumedAuthTicketNonces<Test>>::put(vec![b"alice_auth_ticket_nonce".to_vec()]);

        // Prepare the test input.
        let input = make_input(
            b"alice_pk",
            b"new_alice_auth_ticket_nonce",
            b"should_be_valid",
        );

        // Set up mock expectations for Bioauth::on_initialize.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set()
                .once()
                .with(predicate::eq(vec![]))
                .return_const(());
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().once().with().return_const(expires_at);
        });

        // Run the expiration processing for the previous authentication.
        Bioauth::on_initialize(block_to_process_moment(expires_at));

        // Set up mock expectations for Bioauth::authenticate.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set()
                .once()
                .with(predicate::eq(vec![b"alice_pk".to_vec()]))
                .return_const(());
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().once().with().return_const(expires_at);
        });

        // Make test and ensure that authentication call is processed successfully.
        assert_ok!(Bioauth::authenticate(Origin::none(), input));

        // Ensure that the last authentication has been added to the ActiveAuthentications.
        assert_eq!(
            Bioauth::active_authentications(),
            vec![Authentication {
                public_key: b"alice_pk".to_vec(),
                expires_at: expires_at + AUTHENTICATIONS_EXPIRE_AFTER,
            }]
        );

        // Ensure that the current state of ConsumedAuthTicketNonces has nonces from both authentications.
        assert_eq!(
            Bioauth::consumed_auth_ticket_nonces(),
            vec![
                b"alice_auth_ticket_nonce".to_vec(),
                b"new_alice_auth_ticket_nonce".to_vec()
            ]
        );
    });
}

/// This test prevents authentication call with invalid signature.
#[test]
fn authentication_with_invalid_signature() {
    new_test_ext().execute_with(|| {
        // Prepare test input.
        let input = make_input(b"qwe", b"rty", b"invalid");

        // Make test.
        assert_noop!(
            Bioauth::authenticate(Origin::none(), input),
            Error::<Test>::AuthTicketSignatureInvalid
        );
    });
}

/// This test prevents authentication call with conflicting nonces.
#[test]
fn authentication_with_conlicting_nonce() {
    new_test_ext().execute_with(|| {
        // Set up mock expectations.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set()
                .once()
                .with(predicate::eq(vec![b"pk1".to_vec()]))
                .return_const(());
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().once().return_const(CHAIN_START);
        });

        // Prepare the test precondition.
        let precondition_input = make_input(b"pk1", b"conflict!", b"should_be_valid");
        assert_ok!(Bioauth::authenticate(Origin::none(), precondition_input));

        // Prepare test input.
        let input = make_input(b"pk2", b"conflict!", b"should_be_valid");

        // Make test and ensure the expected error is thrown when no value is present.
        assert_noop!(
            Bioauth::authenticate(Origin::none(), input),
            Error::<Test>::NonceAlreadyUsed,
        );
    });
}

/// This test prevents authentication call with conflicting nonces when previous
/// authentication has been expired.
#[test]
fn authentication_with_conlicting_nonce_after_expiration() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let expires_at = CHAIN_START + 2 * SLOT_DURATION;
        <ActiveAuthentications<Test>>::put(vec![Authentication {
            public_key: b"alice_pk".to_vec(),
            expires_at,
        }]);
        <ConsumedAuthTicketNonces<Test>>::put(vec![b"alice_auth_ticket_nonce".to_vec()]);

        // Set up mock expectations for Bioauth::on_initialize.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set()
                .once()
                .with(predicate::eq(vec![]))
                .return_const(());
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().once().with().return_const(expires_at);
        });

        Bioauth::on_initialize(block_to_process_moment(expires_at));

        // Prepare the test input.
        let input = make_input(b"alice_pk", b"alice_auth_ticket_nonce", b"should_be_valid");

        // Ensure the expected error is thrown when conflicting nonce is used.
        assert_noop!(
            Bioauth::authenticate(Origin::none(), input),
            Error::<Test>::NonceAlreadyUsed,
        );
    });
}

/// This test prevents authentication call with conflicting public keys.
#[test]
fn authentication_with_concurrent_conlicting_public_keys() {
    new_test_ext().execute_with(|| {
        // Set up mock expectations.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set()
                .once()
                .with(predicate::eq(vec![b"conflict!".to_vec()]))
                .return_const(());
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().once().with().return_const(CHAIN_START);
        });

        // Prepare the test precondition.
        let precondition_input = make_input(b"conflict!", b"nonce1", b"should_be_valid");
        assert_ok!(Bioauth::authenticate(Origin::none(), precondition_input));

        // Prepare test input.
        let input = make_input(b"conflict!", b"nonce2", b"should_be_valid");

        // Make test and ensure the expected error is thrown when conflicting public keys is used.
        assert_noop!(
            Bioauth::authenticate(Origin::none(), input),
            Error::<Test>::PublicKeyAlreadyUsed,
        );
    });
}

/// This test verifies SignedExt logic for transaction processing with empty state.
#[test]
fn signed_ext_check_bioauth_tx_permits_empty_state() {
    new_test_ext().execute_with(|| {
        // Prepare test input.
        let input = make_input(b"qwe", b"rty", b"should_be_valid");
        let expected_tag = AuthTicket {
            public_key: b"qwe".to_vec(),
            nonce: b"rty".to_vec(),
        };

        // Make test.
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

/// This test verifies SignedExt logic for transaction processing that contains invalid signature.
#[test]
fn signed_ext_check_bioauth_tx_deny_invalid_signature() {
    new_test_ext().execute_with(|| {
        // Prepare test input.
        let input = make_input(b"qwe", b"rty", b"invalid");

        // Make test.
        let call = <pallet_bioauth::Call<Test>>::authenticate(input).into();
        let info = DispatchInfo::default();

        assert_eq!(
            CheckBioauthTx::<Test>(PhantomData).validate(&1, &call, &info, 1),
            InvalidTransaction::Custom(b's').into()
        );
    })
}

/// This test verifies SignedExt logic for transaction processing with conflicting nonce.
#[test]
fn signed_ext_check_bioauth_tx_denies_conlicting_nonce() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let precondition_input = make_input(b"pk1", b"conflict!", b"should_be_valid");

        // Set up mock expectations for Bioauth::on_initialize.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set()
                .once()
                .with(predicate::eq(vec![b"pk1".to_vec()]))
                .return_const(());
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().once().with().return_const(CHAIN_START);
        });

        assert_ok!(Bioauth::authenticate(Origin::none(), precondition_input));

        // Prepare test input.
        let input = make_input(b"pk2", b"conflict!", b"should_be_valid");

        // Make test.
        let call = <pallet_bioauth::Call<Test>>::authenticate(input).into();
        let info = DispatchInfo::default();

        assert_eq!(
            CheckBioauthTx::<Test>(PhantomData).validate(&1, &call, &info, 1),
            InvalidTransaction::Custom(b'c').into()
        );
    })
}

/// This test verifies SignedExt logic for transaction processing with conflicting public keys.
#[test]
fn signed_ext_check_bioauth_tx_denies_conflicting_public_keys() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let precondition_input = make_input(b"conflict!", b"nonce1", b"should_be_valid");

        // Set up mock expectations for Bioauth::on_initialize.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set()
                .once()
                .with(predicate::eq(vec![b"conflict!".to_vec()]))
                .return_const(());
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().once().with().return_const(CHAIN_START);
        });

        assert_ok!(Bioauth::authenticate(Origin::none(), precondition_input));

        // Prepare test input.
        let input = make_input(b"conflict!", b"nonce2", b"should_be_valid");

        // Make test.
        let call = <pallet_bioauth::Call<Test>>::authenticate(input).into();
        let info = DispatchInfo::default();

        assert_eq!(
            CheckBioauthTx::<Test>(PhantomData).validate(&1, &call, &info, 1),
            InvalidTransaction::Custom(b'c').into()
        );
    })
}

/// This test verifies that genesis initialization properly assignes the state and invokes
/// the validators set init.
#[test]
fn genesis_build() {
    // Prepare some sample data and a config.
    let consumed_auth_ticket_nonces = vec![b"nonce1".to_vec(), b"nonce2".to_vec()];
    let active_authentications = vec![
        Authentication {
            public_key: b"key1".to_vec(),
            expires_at: 123,
        },
        Authentication {
            public_key: b"key2".to_vec(),
            expires_at: 456,
        },
    ];
    let config = pallet_bioauth::GenesisConfig {
        robonode_public_key: MockVerifier,
        consumed_auth_ticket_nonces: consumed_auth_ticket_nonces.clone(),
        active_authentications: active_authentications.clone(),
    };

    // Set up mock expectations for validators set initialization.
    with_mock_validator_set_updater(|mock| {
        mock.expect_init_validators_set()
            .once()
            .with(predicate::eq(vec![b"key1".to_vec(), b"key2".to_vec()]))
            .return_const(());
    });

    // Build the state from the config.
    new_test_ext_with(config).execute_with(move || {
        // Assert the validator set init invocation.
        with_mock_validator_set_updater(|mock| mock.checkpoint());

        // Assert the state.
        assert_eq!(Bioauth::robonode_public_key(), MockVerifier);
        assert_eq!(
            Bioauth::consumed_auth_ticket_nonces(),
            consumed_auth_ticket_nonces
        );
        assert_eq!(Bioauth::active_authentications(), active_authentications);
    })
}
