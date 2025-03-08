// Allow simple integer arithmetic in tests.
#![allow(clippy::arithmetic_side_effects)]

use std::ops::Div;

use frame_support::{
    assert_err, assert_noop, assert_ok, assert_storage_noop, dispatch::DispatchInfo,
    pallet_prelude::*, traits::ConstU32, BoundedVec,
};
use mockall::predicate;

use crate::{self as pallet_bioauth, mock::testing::*, *};

/// An exact value of January 1st 2021 in UNIX time milliseconds.
const JANUARY_1_2021: UnixMilliseconds = 1609459200 * 1000;

/// The time at which the test chain "started".
const CHAIN_START: UnixMilliseconds = JANUARY_1_2021;

fn make_input(
    public_key: ValidatorPublicKey,
    nonce: &[u8],
    signature: &[u8],
) -> pallet_bioauth::Authenticate<MockOpaqueAuthTicket, Vec<u8>> {
    pallet_bioauth::Authenticate {
        ticket: MockOpaqueAuthTicket(AuthTicket {
            public_key,
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

fn make_bounded_active_authentications(
    authentications: Vec<Authentication<ValidatorPublicKey, UnixMilliseconds>>,
) -> BoundedVec<Authentication<ValidatorPublicKey, UnixMilliseconds>, ConstU32<MAX_AUTHENTICATIONS>>
{
    BoundedVec::<_, ConstU32<MAX_AUTHENTICATIONS>>::try_from(authentications).unwrap()
}

fn make_bounded_consumed_auth_nonces(
    auth_nonces: Vec<Vec<u8>>,
) -> BoundedVec<pallet_bioauth::BoundedAuthTicketNonce, ConstU32<MAX_NONCES>> {
    BoundedVec::<_, ConstU32<MAX_NONCES>>::try_from(
        auth_nonces
            .iter()
            .cloned()
            .map(|nonce| BoundedAuthTicketNonce::try_from(nonce).unwrap())
            .collect::<Vec<_>>(),
    )
    .unwrap()
}

fn bounded(data: &[u8]) -> [u8; 32] {
    let mut bounded = [0u8; 32];
    bounded[..data.len()].copy_from_slice(data);
    bounded
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
        let input = make_input(bounded(b"qwe"), b"rty", b"should_be_valid");
        let current_moment = CHAIN_START + 2 * SLOT_DURATION;
        let expires_at = current_moment + AUTHENTICATIONS_EXPIRE_AFTER;

        // Set up mock expectations.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set()
                .once()
                .with(predicate::eq(vec![bounded(b"qwe")]))
                .return_const(());
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().once().with().return_const(current_moment);
        });
        with_mock_before_auth_hook_provider(|mock| {
            mock.expect_hook()
                .once()
                .withf(move |authentication| {
                    authentication
                        == &Authentication {
                            public_key: bounded(b"qwe"),
                            expires_at,
                        }
                })
                .return_const(Ok(()));
        });
        with_mock_after_auth_hook_provider(|mock| {
            mock.expect_hook()
                .once()
                .with(predicate::eq(()))
                .return_const(());
        });

        // Set block number to enable events.
        System::set_block_number(1);

        // Ensure that authentication call is processed successfully.
        assert_ok!(Bioauth::authenticate(RuntimeOrigin::none(), input));

        // Ensure that the state of ActiveAuthentications has been updated.
        assert_eq!(
            Bioauth::active_authentications(),
            vec![Authentication {
                public_key: bounded(b"qwe"),
                expires_at,
            }]
        );
        // Ensure that the state of ConsumedAuthTicketNonces has been updated.
        assert_eq!(
            Bioauth::consumed_auth_ticket_nonces().into_inner(),
            vec![b"rty".to_vec()]
        );

        System::assert_has_event(RuntimeEvent::Bioauth(Event::NewAuthentication {
            authentication: Authentication {
                public_key: bounded(b"qwe"),
                expires_at,
            },
        }));
    });
}

/// This test verifies that authentication expiration logic works correctly after getting
/// the block exactly at the moment the authentication becomes expired.
#[test]
fn authentication_expires_exactly_at_the_moment() {
    new_test_ext().execute_with(|| {
        // Prepare the test preconditions.
        let expires_at = CHAIN_START + 2 * SLOT_DURATION;

        let bounded_active_authentications =
            make_bounded_active_authentications(vec![Authentication {
                public_key: bounded(b"alice_pk"),
                expires_at,
            }]);

        let bounded_consumed_auth_ticket_nonces =
            make_bounded_consumed_auth_nonces(vec![b"alice_auth_ticket_nonce".to_vec()]);

        <ActiveAuthentications<Test>>::put(bounded_active_authentications);
        <ConsumedAuthTicketNonces<Test>>::put(bounded_consumed_auth_ticket_nonces);

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

        // Declare that before/after auth hooks must not run.
        with_mock_before_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });
        with_mock_after_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });

        // Set block number to enable events.
        System::set_block_number(1);

        // Process the block that certainly has to expire the authentication.
        Bioauth::on_initialize(block_to_process_moment(expires_at));

        // Ensure that authentication expires.
        assert_eq!(Bioauth::active_authentications(), vec![]);
        // Ensure that nonce didn't go anywhere as it's still listed as blocked.
        assert_eq!(
            Bioauth::consumed_auth_ticket_nonces().into_inner(),
            vec![b"alice_auth_ticket_nonce".to_vec()]
        );

        System::assert_has_event(RuntimeEvent::Bioauth(Event::AuthenticationsExpired {
            expired: vec![Authentication {
                public_key: bounded(b"alice_pk"),
                expires_at,
            }],
        }));
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

        let bounded_active_authentications =
            make_bounded_active_authentications(vec![Authentication {
                public_key: bounded(b"alice_pk"),
                expires_at,
            }]);

        let bounded_consumed_auth_ticket_nonces =
            make_bounded_consumed_auth_nonces(vec![b"alice_auth_ticket_nonce".to_vec()]);

        <ActiveAuthentications<Test>>::put(bounded_active_authentications);
        <ConsumedAuthTicketNonces<Test>>::put(bounded_consumed_auth_ticket_nonces);

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

        // Declare that before/after auth hooks must not run.
        with_mock_before_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });
        with_mock_after_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });

        // Set block number to enable events.
        System::set_block_number(1);

        // Process the block that certainly has to expire the authentication.
        Bioauth::on_initialize(block_to_process_moment(expires_at));

        // Ensure that authentication expires.
        assert_eq!(Bioauth::active_authentications(), vec![]);
        // Ensure that nonce didn't go anywhere as it's still listed as blocked.
        assert_eq!(
            Bioauth::consumed_auth_ticket_nonces().into_inner(),
            vec![b"alice_auth_ticket_nonce".to_vec()]
        );

        System::assert_has_event(RuntimeEvent::Bioauth(Event::AuthenticationsExpired {
            expired: vec![Authentication {
                public_key: bounded(b"alice_pk"),
                expires_at,
            }],
        }));
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

        let authentication = Authentication {
            public_key: bounded(b"alice_pk"),
            expires_at,
        };

        let nonce = b"alice_auth_ticket_nonce".to_vec();

        let bounded_authentication =
            make_bounded_active_authentications(vec![authentication.clone()]);

        let bounded_nonce = make_bounded_consumed_auth_nonces(vec![nonce.clone()]);

        // Declare that before/after auth hooks must not run.
        with_mock_before_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });
        with_mock_after_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });

        <ActiveAuthentications<Test>>::put(bounded_authentication);
        <ConsumedAuthTicketNonces<Test>>::put(bounded_nonce);

        loop {
            // Set up mock expectations.
            with_mock_current_moment_provider(|mock| {
                mock.expect_now().once().with().return_const(current_moment);
            });

            assert_storage_noop!(Bioauth::on_initialize(System::block_number()));

            // Ensure that authentication is still active.
            assert_eq!(
                Bioauth::active_authentications(),
                vec![authentication.clone()]
            );
            // Ensure that nonce didn't go anywhere and it's still listed as blocked.
            assert_eq!(
                Bioauth::consumed_auth_ticket_nonces().into_inner(),
                vec![nonce.clone()]
            );

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
        assert_eq!(
            Bioauth::consumed_auth_ticket_nonces().into_inner(),
            vec![nonce]
        );

        System::assert_has_event(RuntimeEvent::Bioauth(Event::AuthenticationsExpired {
            expired: vec![Authentication {
                public_key: bounded(b"alice_pk"),
                expires_at,
            }],
        }));
    });
}

/// This test verifies that authentication call works correctly when a previous
/// authentication has been expired.
#[test]
fn authentication_when_previous_one_has_been_expired() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let expires_at = CHAIN_START + 2 * SLOT_DURATION;

        let bounded_active_authentications =
            make_bounded_active_authentications(vec![Authentication {
                public_key: bounded(b"alice_pk"),
                expires_at,
            }]);

        let bounded_consumed_auth_ticket_nonces =
            make_bounded_consumed_auth_nonces(vec![b"alice_auth_ticket_nonce".to_vec()]);

        <ActiveAuthentications<Test>>::put(bounded_active_authentications);
        <ConsumedAuthTicketNonces<Test>>::put(bounded_consumed_auth_ticket_nonces);

        // Prepare the test input.
        let input = make_input(
            bounded(b"alice_pk"),
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
        with_mock_before_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });
        with_mock_after_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });

        // Set block number to enable events.
        System::set_block_number(1);

        // Run the expiration processing for the previous authentication.
        Bioauth::on_initialize(block_to_process_moment(expires_at));

        System::assert_has_event(RuntimeEvent::Bioauth(Event::AuthenticationsExpired {
            expired: vec![Authentication {
                public_key: bounded(b"alice_pk"),
                expires_at,
            }],
        }));

        // Set up mock expectations for Bioauth::authenticate.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set()
                .once()
                .with(predicate::eq(vec![bounded(b"alice_pk")]))
                .return_const(());
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().once().with().return_const(expires_at);
        });
        with_mock_before_auth_hook_provider(|mock| {
            mock.expect_hook()
                .once()
                .with(predicate::function(move |authentication| {
                    authentication
                        == &Authentication {
                            public_key: bounded(b"alice_pk"),
                            expires_at: expires_at + AUTHENTICATIONS_EXPIRE_AFTER,
                        }
                }))
                .return_const(Ok(()));
        });
        with_mock_after_auth_hook_provider(|mock| {
            mock.expect_hook()
                .once()
                .with(predicate::eq(()))
                .return_const(());
        });

        // Make test and ensure that authentication call is processed successfully.
        assert_ok!(Bioauth::authenticate(RuntimeOrigin::none(), input));

        // Ensure that the last authentication has been added to the ActiveAuthentications.
        assert_eq!(
            Bioauth::active_authentications(),
            vec![Authentication {
                public_key: bounded(b"alice_pk"),
                expires_at: expires_at + AUTHENTICATIONS_EXPIRE_AFTER,
            }]
        );

        // Ensure that the current state of ConsumedAuthTicketNonces has nonces from both authentications.
        assert_eq!(
            Bioauth::consumed_auth_ticket_nonces().into_inner(),
            vec![
                b"alice_auth_ticket_nonce".to_vec(),
                b"new_alice_auth_ticket_nonce".to_vec()
            ]
        );

        System::assert_has_event(RuntimeEvent::Bioauth(Event::NewAuthentication {
            authentication: Authentication {
                public_key: bounded(b"alice_pk"),
                expires_at: expires_at + AUTHENTICATIONS_EXPIRE_AFTER,
            },
        }));
    });
}

/// This test prevents authentication when the authentications limit has been reached as `BoundedVec`.
#[test]
fn too_many_authentications() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let expires_at = CHAIN_START + 2 * SLOT_DURATION;

        let mut active_authentications = vec![];

        for authentication in 0..MAX_AUTHENTICATIONS {
            let public_key = bounded(format!("pk_{authentication}").as_bytes());
            active_authentications.push(Authentication {
                public_key,
                expires_at,
            });
        }

        let bounded_active_authentications =
            make_bounded_active_authentications(active_authentications);

        <ActiveAuthentications<Test>>::put(bounded_active_authentications);

        // Prepare the test input.
        let input = make_input(
            bounded(b"alice_pk"),
            b"alice_auth_ticket_nonce",
            b"should_be_valid",
        );

        // Set up mock expectations.
        let current_moment = CHAIN_START + SLOT_DURATION;

        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set().never();
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().once().with().return_const(current_moment);
        });
        with_mock_before_auth_hook_provider(|mock| {
            mock.expect_hook()
                .once()
                .with(predicate::function(move |authentication| {
                    authentication
                        == &Authentication {
                            public_key: bounded(b"alice_pk"),
                            expires_at: current_moment + AUTHENTICATIONS_EXPIRE_AFTER,
                        }
                }))
                .return_const(Ok(()));
        });
        with_mock_after_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });

        // Make test.
        assert_noop!(
            Bioauth::authenticate(RuntimeOrigin::none(), input),
            Error::<Test>::TooManyAuthentications
        );
    });
}

/// This test prevents authentication when the consumed auth ticket nonces
/// limit has been reached as `BoundedVec`.
#[test]
fn too_many_nonces() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let mut consumed_auth_ticket_nonces = vec![];

        for nonce in 0..MAX_NONCES {
            consumed_auth_ticket_nonces
                .push(format!("auth_ticket_nonce_{nonce}").as_bytes().to_vec());
        }

        let bounded_consumed_auth_ticket_nonces =
            make_bounded_consumed_auth_nonces(consumed_auth_ticket_nonces);

        <ConsumedAuthTicketNonces<Test>>::put(bounded_consumed_auth_ticket_nonces);

        // Prepare the test input.
        let input = make_input(
            bounded(b"alice_pk"),
            b"alice_auth_ticket_nonce",
            b"should_be_valid",
        );

        // Set up mock expectations.
        let current_moment = CHAIN_START + SLOT_DURATION;

        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set().never();
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().once().with().return_const(current_moment);
        });
        with_mock_before_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });
        with_mock_after_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });

        // Make test.
        assert_noop!(
            Bioauth::authenticate(RuntimeOrigin::none(), input),
            Error::<Test>::TooManyNonces
        );
    });
}

/// This test prevents authentication when the number of bytes at the nonce has reached the limit.
#[test]
fn too_many_bytes_in_nonce() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let invalid_nonce = vec![1u8; (AUTH_TICKET_NONCE_MAX_BYTES + 1).try_into().unwrap()];

        // Prepare the test input.
        let input = make_input(bounded(b"alice_pk"), &invalid_nonce, b"should_be_valid");

        // Set up mock expectations.
        let current_moment = CHAIN_START + SLOT_DURATION;

        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set().never();
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().once().with().return_const(current_moment);
        });
        with_mock_before_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });
        with_mock_after_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });

        // Make test.
        assert_noop!(
            Bioauth::authenticate(RuntimeOrigin::none(), input),
            Error::<Test>::TooManyBytesInNonce
        );
    });
}

/// This test prevents authentication call with invalid signature.
#[test]
fn authentication_with_invalid_signature() {
    new_test_ext().execute_with(|| {
        // Prepare test input.
        let input = make_input(bounded(b"qwe"), b"rty", b"invalid");

        // Make test.
        assert_noop!(
            Bioauth::authenticate(RuntimeOrigin::none(), input),
            Error::<Test>::AuthTicketSignatureInvalid
        );
    });
}

/// This test prevents authentication call with conflicting nonces.
#[test]
fn authentication_with_conlicting_nonce() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let expires_at = CHAIN_START + 2 * SLOT_DURATION;

        let bounded_active_authentications =
            make_bounded_active_authentications(vec![Authentication {
                public_key: bounded(b"pk1"),
                expires_at,
            }]);

        let bounded_consumed_auth_ticket_nonces =
            make_bounded_consumed_auth_nonces(vec![b"conflict!".to_vec()]);

        <ActiveAuthentications<Test>>::put(bounded_active_authentications);
        <ConsumedAuthTicketNonces<Test>>::put(bounded_consumed_auth_ticket_nonces);

        // Set up mock expectations.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set().never();
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().never();
        });
        with_mock_before_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });
        with_mock_after_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });

        // Prepare test input.
        let input = make_input(bounded(b"pk2"), b"conflict!", b"should_be_valid");

        // Make test and ensure the expected error is thrown when no value is present.
        assert_noop!(
            Bioauth::authenticate(RuntimeOrigin::none(), input),
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

        let bounded_active_authentications =
            make_bounded_active_authentications(vec![Authentication {
                public_key: bounded(b"alice_pk"),
                expires_at,
            }]);

        let bounded_consumed_auth_ticket_nonces =
            make_bounded_consumed_auth_nonces(vec![b"alice_auth_ticket_nonce".to_vec()]);

        <ActiveAuthentications<Test>>::put(bounded_active_authentications);
        <ConsumedAuthTicketNonces<Test>>::put(bounded_consumed_auth_ticket_nonces);

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
        with_mock_before_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });
        with_mock_after_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });

        Bioauth::on_initialize(block_to_process_moment(expires_at));

        // Set up mock expectations for Bioauth::authenticate.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set().never();
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().never();
        });
        with_mock_before_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });
        with_mock_after_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });

        // Prepare the test input.
        let input = make_input(
            bounded(b"alice_pk"),
            b"alice_auth_ticket_nonce",
            b"should_be_valid",
        );

        // Ensure the expected error is thrown when conflicting nonce is used.
        assert_noop!(
            Bioauth::authenticate(RuntimeOrigin::none(), input),
            Error::<Test>::NonceAlreadyUsed,
        );
    });
}

/// This test prevents authentication call with conflicting public keys.
#[test]
fn authentication_with_concurrent_conflicting_public_keys() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let expires_at = CHAIN_START + 2 * SLOT_DURATION;

        let bounded_active_authentications =
            make_bounded_active_authentications(vec![Authentication {
                public_key: bounded(b"conflict!"),
                expires_at,
            }]);

        let bounded_consumed_auth_ticket_nonces =
            make_bounded_consumed_auth_nonces(vec![b"nonce1".to_vec()]);

        <ActiveAuthentications<Test>>::put(bounded_active_authentications);
        <ConsumedAuthTicketNonces<Test>>::put(bounded_consumed_auth_ticket_nonces);

        // Set up mock expectations.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set().never();
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().never();
        });
        with_mock_before_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });
        with_mock_after_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });

        // Prepare test input.
        let input = make_input(bounded(b"conflict!"), b"nonce2", b"should_be_valid");

        // Make test and ensure the expected error is thrown when conflicting public keys is used.
        assert_noop!(
            Bioauth::authenticate(RuntimeOrigin::none(), input),
            Error::<Test>::PublicKeyAlreadyUsed,
        );
    });
}

/// This test verifies that before auth hook can deny the authentication
/// and the resulting state is proper.
#[test]
fn authentication_denied_by_before_hook() {
    new_test_ext().execute_with(|| {
        // Prepare test input.
        let input = make_input(bounded(b"qwe"), b"rty", b"should_be_valid");
        let current_moment = CHAIN_START + 2 * SLOT_DURATION;
        let expires_at = current_moment + AUTHENTICATIONS_EXPIRE_AFTER;

        // Set up mock expectations.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set().never();
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().once().with().return_const(current_moment);
        });
        with_mock_before_auth_hook_provider(|mock| {
            mock.expect_hook()
                .once()
                .withf(move |authentication| {
                    authentication
                        == &Authentication {
                            public_key: bounded(b"qwe"),
                            expires_at,
                        }
                })
                .return_const(Err(sp_runtime::DispatchError::CannotLookup));
        });
        with_mock_after_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });

        // Ensure that authentication call is denied.
        assert_err!(
            Bioauth::authenticate(RuntimeOrigin::none(), input),
            sp_runtime::DispatchError::CannotLookup
        );

        // Ensure that the state of ActiveAuthentications has not been updated.
        assert_eq!(Bioauth::active_authentications(), vec![]);

        // Ensure that the state of ConsumedAuthTicketNonces has not been updated.
        assert_eq!(Bioauth::consumed_auth_ticket_nonces(), vec![]);
    });
}

/// This test verifies that the [`set_robonode_public_key`] updates the robonode public key properly.
#[test]
fn set_robonode_public_key_updates_key() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let expires_at = CHAIN_START + 2 * SLOT_DURATION;
        let bounded_active_authentications =
            make_bounded_active_authentications(vec![Authentication {
                public_key: bounded(b"key1"),
                expires_at,
            }]);

        let bounded_consumed_auth_ticket_nonces =
            make_bounded_consumed_auth_nonces(vec![b"nonce1".to_vec()]);

        <ActiveAuthentications<Test>>::put(bounded_active_authentications);
        <ConsumedAuthTicketNonces<Test>>::put(bounded_consumed_auth_ticket_nonces.clone());

        // Check the test precondition.
        assert_eq!(Bioauth::robonode_public_key(), MockVerifier::A);

        // Prepare test input.
        let input = MockVerifier::B;

        // Execute the key change.
        assert_ok!(Bioauth::set_robonode_public_key(
            RuntimeOrigin::root(),
            input
        ));

        // Ensure the key has changed.
        assert_eq!(Bioauth::robonode_public_key(), MockVerifier::B);

        // Ensure the active authentications are cleared.
        assert_eq!(<ActiveAuthentications<Test>>::get(), vec![]);

        // Ensure that the auth ticket nonces are *not* cleared.
        assert_eq!(
            <ConsumedAuthTicketNonces<Test>>::get(),
            bounded_consumed_auth_ticket_nonces
        );
    });
}

/// This test verifies that the [`set_robonode_public_key`] checks the origin.
#[test]
fn set_robonode_public_key_checks_the_origin() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let expires_at = CHAIN_START + 2 * SLOT_DURATION;
        let bounded_active_authentications =
            make_bounded_active_authentications(vec![Authentication {
                public_key: bounded(b"key1"),
                expires_at,
            }]);

        let bounded_consumed_auth_ticket_nonces =
            make_bounded_consumed_auth_nonces(vec![b"nonce1".to_vec()]);

        <ActiveAuthentications<Test>>::put(bounded_active_authentications.clone());
        <ConsumedAuthTicketNonces<Test>>::put(bounded_consumed_auth_ticket_nonces.clone());

        // Check the test precondition.
        assert_eq!(Bioauth::robonode_public_key(), MockVerifier::A);

        // Prepare test input.
        let input = MockVerifier::B;

        // Attempt key changes with various origins.
        assert_err!(
            Bioauth::set_robonode_public_key(RuntimeOrigin::none(), input.clone()),
            sp_runtime::DispatchError::BadOrigin
        );
        assert_err!(
            Bioauth::set_robonode_public_key(RuntimeOrigin::signed(123), input),
            sp_runtime::DispatchError::BadOrigin
        );

        // Ensure that the key has not changed.
        assert_eq!(Bioauth::robonode_public_key(), MockVerifier::A);

        // Ensure that the active authentications are *not* cleared.
        assert_eq!(
            <ActiveAuthentications<Test>>::get(),
            bounded_active_authentications
        );

        // Ensure that the auth ticket nonces are *not* cleared.
        assert_eq!(
            <ConsumedAuthTicketNonces<Test>>::get(),
            bounded_consumed_auth_ticket_nonces
        );
    });
}

/// This test verifies `SignedExt` logic for transaction processing with empty state.
#[test]
fn signed_ext_check_bioauth_tx_permits_empty_state() {
    new_test_ext().execute_with(|| {
        // Prepare test input.
        let input = make_input(bounded(b"qwe"), b"rty", b"should_be_valid");
        let expected_tag = AuthTicket {
            public_key: bounded(b"qwe"),
            nonce: b"rty".to_vec(),
        };

        // Make test.
        let call = pallet_bioauth::Call::authenticate { req: input }.into();
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

/// This test verifies `SignedExt` logic for transaction processing that contains invalid signature.
#[test]
fn signed_ext_check_bioauth_tx_deny_invalid_signature() {
    new_test_ext().execute_with(|| {
        // Prepare test input.
        let input = make_input(bounded(b"qwe"), b"rty", b"invalid");

        // Make test.
        let call = pallet_bioauth::Call::authenticate { req: input }.into();
        let info = DispatchInfo::default();

        assert_eq!(
            CheckBioauthTx::<Test>(PhantomData).validate(&1, &call, &info, 1),
            InvalidTransaction::BadProof.into()
        );
    })
}

/// This test verifies `SignedExt` logic for transaction processing with conflicting nonce.
#[test]
fn signed_ext_check_bioauth_tx_denies_conlicting_nonce() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let expires_at = CHAIN_START + 2 * SLOT_DURATION;

        let bounded_active_authentications =
            make_bounded_active_authentications(vec![Authentication {
                public_key: bounded(b"pk1"),
                expires_at,
            }]);

        let bounded_consumed_auth_ticket_nonces =
            make_bounded_consumed_auth_nonces(vec![b"conflict!".to_vec()]);

        <ActiveAuthentications<Test>>::put(bounded_active_authentications);
        <ConsumedAuthTicketNonces<Test>>::put(bounded_consumed_auth_ticket_nonces);

        // Set up mock expectations for the precondition Bioauth::authenticate.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set().never();
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().never();
        });
        with_mock_before_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });
        with_mock_after_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });

        // Prepare test input.
        let input = make_input(bounded(b"pk2"), b"conflict!", b"should_be_valid");

        // Make test.
        let call = pallet_bioauth::Call::authenticate { req: input }.into();
        let info = DispatchInfo::default();

        assert_eq!(
            CheckBioauthTx::<Test>(PhantomData).validate(&1, &call, &info, 1),
            InvalidTransaction::Stale.into()
        );
    })
}

/// This test verifies `SignedExt` logic for transaction processing with conflicting public keys.
#[test]
fn signed_ext_check_bioauth_tx_denies_conflicting_public_keys() {
    new_test_ext().execute_with(|| {
        // Prepare the test precondition.
        let expires_at = CHAIN_START + 2 * SLOT_DURATION;

        let bounded_active_authentications =
            make_bounded_active_authentications(vec![Authentication {
                public_key: bounded(b"conflict!"),
                expires_at,
            }]);

        let bounded_consumed_auth_ticket_nonces =
            make_bounded_consumed_auth_nonces(vec![b"nonce1".to_vec()]);

        <ActiveAuthentications<Test>>::put(bounded_active_authentications);
        <ConsumedAuthTicketNonces<Test>>::put(bounded_consumed_auth_ticket_nonces);

        // Set up mock expectations for Bioauth::authenticate.
        with_mock_validator_set_updater(|mock| {
            mock.expect_update_validators_set().never();
        });
        with_mock_current_moment_provider(|mock| {
            mock.expect_now().never();
        });
        with_mock_before_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });
        with_mock_after_auth_hook_provider(|mock| {
            mock.expect_hook().never();
        });

        // Prepare test input.
        let input = make_input(bounded(b"conflict!"), b"nonce2", b"should_be_valid");

        // Make test.
        let call = pallet_bioauth::Call::authenticate { req: input }.into();
        let info = DispatchInfo::default();

        assert_eq!(
            CheckBioauthTx::<Test>(PhantomData).validate(&1, &call, &info, 1),
            InvalidTransaction::Future.into()
        );
    })
}

/// This test verifies that genesis initialization properly assigns the state and invokes
/// the validators set init.
#[test]
fn genesis_build() {
    // Prepare some sample data and a config.
    let consumed_auth_ticket_nonces = BoundedVec::try_from(vec![
        BoundedVec::try_from(b"nonce1".to_vec()).unwrap(),
        BoundedVec::try_from(b"nonce2".to_vec()).unwrap(),
    ])
    .unwrap();
    let active_authentications = BoundedVec::try_from(vec![
        Authentication {
            public_key: bounded(b"key1"),
            expires_at: 123,
        },
        Authentication {
            public_key: bounded(b"key2"),
            expires_at: 456,
        },
    ])
    .unwrap();
    let config = pallet_bioauth::GenesisConfig {
        robonode_public_key: MockVerifier::A,
        consumed_auth_ticket_nonces: consumed_auth_ticket_nonces.clone(),
        active_authentications: active_authentications.clone(),
    };

    // Set up mock expectations for validators set initialization.
    with_mock_validator_set_updater(|mock| {
        mock.expect_init_validators_set()
            .once()
            .with(predicate::eq(vec![bounded(b"key1"), bounded(b"key2")]))
            .return_const(());
    });

    // Build the state from the config.
    new_test_ext_with(config).execute_with(move || {
        // Assert the validator set init invocation.
        with_mock_validator_set_updater(|mock| mock.checkpoint());

        // Assert the state.
        assert_eq!(Bioauth::robonode_public_key(), MockVerifier::A);
        assert_eq!(
            Bioauth::consumed_auth_ticket_nonces(),
            consumed_auth_ticket_nonces
        );
        assert_eq!(Bioauth::active_authentications(), active_authentications);
    })
}
