//! The tests for the pallet.

// Allow simple integer arithmetic in tests.
#![allow(clippy::arithmetic_side_effects)]

use crate::{mock::*, *};

/// This test verifies that basic setup works in the happy path.
#[test]
fn basic_setup_works() {
    new_test_ext().execute_with_ext(|_| {
        assert_eq!(<Total<Test>>::get(), None);
    });
}

/// This test verifies that basic offence report logic works in the happy path
/// by slashing not bootnode validator.
#[test]
fn offence_report_slash_not_bootnode_works() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test input data.
        let offenders = vec![(
            1,
            pallet_humanode_session::Identification::Bioauth(pallet_bioauth::Authentication {
                public_key: 1,
                expires_at: 1000,
            }),
        )];

        // Check test preconditions.
        assert_eq!(<Total<Test>>::get(), None);
        assert_eq!(
            Bioauth::active_authentications(),
            vec![pallet_bioauth::Authentication {
                public_key: 1,
                expires_at: 1000,
            }]
        );
        assert_eq!(Bootnodes::bootnodes(), vec![42]);

        // Set mock expectations.
        let offenders_ctx = MockOffence::offenders_context();
        offenders_ctx
            .expect()
            .once()
            .return_const(offenders.clone());

        // Set block number to enable events.
        System::set_block_number(1);

        // Report offence.
        HumanodeOffences::report_offence(vec![], MockOffence {}).unwrap();

        // Assert state changes.
        assert_eq!(<Total<Test>>::get(), Some(1));
        assert!(Bioauth::active_authentications().is_empty());
        assert_eq!(Bootnodes::bootnodes(), vec![42]);
        System::assert_has_event(RuntimeEvent::HumanodeOffences(Event::ReportedOffence {
            kind: MOCKED_OFFENCE_KIND,
            offenders,
        }));

        // Assert mock invocations.
        offenders_ctx.checkpoint();
    });
}

/// This test verifies that basic offence report logic works in the happy path
/// by not slashing bootnode validator.
#[test]
fn offence_report_not_slash_bootnode_works() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test input data.
        let offenders = vec![(42, pallet_humanode_session::Identification::Bootnode(42))];

        // Check test preconditions.
        assert_eq!(<Total<Test>>::get(), None);
        assert_eq!(
            Bioauth::active_authentications(),
            vec![pallet_bioauth::Authentication {
                public_key: 1,
                expires_at: 1000,
            }]
        );
        assert_eq!(Bootnodes::bootnodes(), vec![42]);

        // Set mock expectations.
        let offenders_ctx = MockOffence::offenders_context();
        offenders_ctx
            .expect()
            .once()
            .return_const(offenders.clone());

        // Set block number to enable events.
        System::set_block_number(1);

        // Report offence.
        HumanodeOffences::report_offence(vec![], MockOffence {}).unwrap();

        // Assert state changes.
        assert_eq!(<Total<Test>>::get(), None);
        assert_eq!(
            Bioauth::active_authentications(),
            vec![pallet_bioauth::Authentication {
                public_key: 1,
                expires_at: 1000,
            }]
        );
        assert_eq!(Bootnodes::bootnodes(), vec![42]);
        System::assert_has_event(RuntimeEvent::HumanodeOffences(Event::ReportedOffence {
            kind: MOCKED_OFFENCE_KIND,
            offenders,
        }));

        // Assert mock invocations.
        offenders_ctx.checkpoint();
    });
}

/// This test verifies that basic offence report logic works in the happy path
/// by not slashing bootnode validator and slashing not bootnode validator.
#[test]
fn offence_report_both_cases_works() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test input data.
        let offenders = vec![
            (
                1,
                pallet_humanode_session::Identification::Bioauth(pallet_bioauth::Authentication {
                    public_key: 1,
                    expires_at: 1000,
                }),
            ),
            (42, pallet_humanode_session::Identification::Bootnode(42)),
        ];

        // Check test preconditions.
        assert_eq!(<Total<Test>>::get(), None);
        assert_eq!(
            Bioauth::active_authentications(),
            vec![pallet_bioauth::Authentication {
                public_key: 1,
                expires_at: 1000,
            }]
        );
        assert_eq!(Bootnodes::bootnodes(), vec![42]);

        // Set mock expectations.
        let offenders_ctx = MockOffence::offenders_context();
        offenders_ctx
            .expect()
            .once()
            .return_const(offenders.clone());

        // Set block number to enable events.
        System::set_block_number(1);

        // Report offence.
        HumanodeOffences::report_offence(vec![], MockOffence {}).unwrap();

        // Assert state changes.
        assert_eq!(<Total<Test>>::get(), Some(1));
        assert!(Bioauth::active_authentications().is_empty());
        assert_eq!(Bootnodes::bootnodes(), vec![42]);
        System::assert_has_event(RuntimeEvent::HumanodeOffences(Event::ReportedOffence {
            kind: MOCKED_OFFENCE_KIND,
            offenders,
        }));

        // Assert mock invocations.
        offenders_ctx.checkpoint();
    });
}
