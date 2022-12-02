//! The tests for the pallet.

use frame_support::{
    assert_noop, assert_ok, assert_storage_noop,
    dispatch::{DispatchClass, DispatchInfo, Pays},
    pallet_prelude::{InvalidTransaction, ValidTransaction},
    unsigned::TransactionValidityError,
    weights::Weight,
};
use mockall::predicate;
use primitives_ethereum::EthereumAddress;
use sp_runtime::{traits::SignedExtension, DispatchError};

use crate::{
    mock::{
        eth, new_test_ext, new_test_ext_with, sig, Balances, EthAddr,
        MockEthereumSignatureVerifier, MockVestingInterface, MockVestingSchedule, RuntimeOrigin,
        Test, TestExternalitiesExt, TokenClaims, FUNDS_CONSUMER, FUNDS_PROVIDER,
    },
    traits::{NoVesting, VestingInterface},
    types::{ClaimInfo, EthereumSignatureMessageParams},
    *,
};

fn pot_account_balance() -> BalanceOf<Test> {
    <CurrencyOf<Test>>::free_balance(&<Test as Config>::PotAccountId::get())
}

fn total_claimable_balance() -> BalanceOf<Test> {
    <TotalClaimable<Test>>::get()
}

fn currency_total_issuance() -> BalanceOf<Test> {
    <CurrencyOf<Test>>::total_issuance()
}

#[test]
fn basic_setup_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check the claims.
        assert_eq!(<Claims<Test>>::get(EthereumAddress::default()), None);
        assert_eq!(
            <Claims<Test>>::get(eth(EthAddr::Existing)),
            Some(ClaimInfo {
                balance: 10,
                vesting: MockVestingSchedule
            })
        );
        assert_eq!(
            <Claims<Test>>::get(eth(EthAddr::SecondExisting)),
            Some(ClaimInfo {
                balance: 20,
                vesting: MockVestingSchedule
            })
        );

        // Check the pot balance.
        assert_eq!(
            pot_account_balance(),
            30 + <CurrencyOf<Test>>::minimum_balance()
        );

        // Check the total claimable balance value.
        assert_eq!(total_claimable_balance(), 30);
    });
}

/// This test verifies that claiming works in the happy path.
#[test]
fn claiming_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(<Claims<Test>>::contains_key(eth(EthAddr::Existing)));
        assert_eq!(Balances::free_balance(42), 0);
        let pot_account_balance_before = pot_account_balance();
        let total_claimable_balance_before = total_claimable_balance();
        let currency_total_issuance_before = currency_total_issuance();

        // Set mock expectations.
        let recover_signer_ctx = MockEthereumSignatureVerifier::recover_signer_context();
        let lock_under_vesting_ctx = MockVestingInterface::lock_under_vesting_context();
        recover_signer_ctx
            .expect()
            .once()
            .with(
                predicate::eq(sig(1)),
                predicate::eq(EthereumSignatureMessageParams {
                    account_id: 42,
                    ethereum_address: eth(EthAddr::Existing),
                }),
            )
            .return_const(Some(eth(EthAddr::Existing)));
        lock_under_vesting_ctx
            .expect()
            .once()
            .with(predicate::eq(42), predicate::eq(10), predicate::always())
            .return_const(Ok(()));

        // Invoke the function under test.
        assert_ok!(TokenClaims::claim(
            RuntimeOrigin::signed(42),
            eth(EthAddr::Existing),
            sig(1)
        ));

        // Assert state changes.
        assert!(!<Claims<Test>>::contains_key(eth(EthAddr::Existing)));
        assert_eq!(Balances::free_balance(42), 10);
        assert_eq!(pot_account_balance_before - pot_account_balance(), 10);
        assert_eq!(
            total_claimable_balance_before - total_claimable_balance(),
            10
        );
        assert_eq!(
            currency_total_issuance_before - currency_total_issuance(),
            0
        );

        // Assert mock invocations.
        recover_signer_ctx.checkpoint();
        lock_under_vesting_ctx.checkpoint();
    });
}

/// This test verifies that claiming does not go through when the ethereum address recovery from
/// the ethereum signature fails.
#[test]
fn claim_eth_signature_recovery_failure() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(<Claims<Test>>::contains_key(eth(EthAddr::Existing)));
        assert_eq!(Balances::free_balance(42), 0);
        let pot_account_balance_before = pot_account_balance();
        let total_claimable_balance_before = total_claimable_balance();
        let currency_total_issuance_before = currency_total_issuance();

        // Set mock expectations.
        let recover_signer_ctx = MockEthereumSignatureVerifier::recover_signer_context();
        let lock_under_vesting_ctx = MockVestingInterface::lock_under_vesting_context();
        recover_signer_ctx
            .expect()
            .once()
            .with(
                predicate::eq(sig(1)),
                predicate::eq(EthereumSignatureMessageParams {
                    account_id: 42,
                    ethereum_address: eth(EthAddr::Existing),
                }),
            )
            .return_const(None);
        lock_under_vesting_ctx.expect().never();

        // Invoke the function under test.
        assert_noop!(
            TokenClaims::claim(RuntimeOrigin::signed(42), eth(EthAddr::Existing), sig(1)),
            <Error<Test>>::InvalidSignature
        );

        // Assert state changes.
        assert!(<Claims<Test>>::contains_key(eth(EthAddr::Existing)));
        assert_eq!(Balances::free_balance(42), 0);
        assert_eq!(pot_account_balance_before - pot_account_balance(), 0);
        assert_eq!(
            total_claimable_balance_before - total_claimable_balance(),
            0
        );
        assert_eq!(
            currency_total_issuance_before - currency_total_issuance(),
            0
        );

        // Assert mock invocations.
        recover_signer_ctx.checkpoint();
        lock_under_vesting_ctx.checkpoint();
    });
}

/// This test verifies that claiming does not go through when the ethereum address recovery from
/// the ethereum signature recoves an address that does not match the expected one.
#[test]
fn claim_eth_signature_recovery_invalid() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(<Claims<Test>>::contains_key(eth(EthAddr::Existing)));
        assert!(!<Claims<Test>>::contains_key(eth(EthAddr::Unknown)));
        assert_eq!(Balances::free_balance(42), 0);
        let pot_account_balance_before = pot_account_balance();
        let total_claimable_balance_before = total_claimable_balance();
        let currency_total_issuance_before = currency_total_issuance();

        // Set mock expectations.
        let recover_signer_ctx = MockEthereumSignatureVerifier::recover_signer_context();
        let lock_under_vesting_ctx = MockVestingInterface::lock_under_vesting_context();
        recover_signer_ctx
            .expect()
            .once()
            .with(
                predicate::eq(sig(1)),
                predicate::eq(EthereumSignatureMessageParams {
                    account_id: 42,
                    ethereum_address: eth(EthAddr::Existing),
                }),
            )
            .return_const(Some(eth(EthAddr::Unknown)));
        lock_under_vesting_ctx.expect().never();

        // Invoke the function under test.
        assert_noop!(
            TokenClaims::claim(RuntimeOrigin::signed(42), eth(EthAddr::Existing), sig(1)),
            <Error<Test>>::InvalidSignature
        );

        // Assert state changes.
        assert!(<Claims<Test>>::contains_key(eth(EthAddr::Existing)));
        assert!(!<Claims<Test>>::contains_key(eth(EthAddr::Unknown)));
        assert_eq!(Balances::free_balance(42), 0);
        assert_eq!(pot_account_balance_before - pot_account_balance(), 0);
        assert_eq!(
            total_claimable_balance_before - total_claimable_balance(),
            0
        );
        assert_eq!(
            currency_total_issuance_before - currency_total_issuance(),
            0
        );

        // Assert mock invocations.
        recover_signer_ctx.checkpoint();
        lock_under_vesting_ctx.checkpoint();
    });
}

/// This test verifies that claiming does end up in a consistent state if the vesting interface call
/// returns an error.
#[test]
fn claim_lock_under_vesting_failure() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(<Claims<Test>>::contains_key(eth(EthAddr::Existing)));
        assert_eq!(Balances::free_balance(42), 0);
        let pot_account_balance_before = pot_account_balance();
        let total_claimable_balance_before = total_claimable_balance();
        let currency_total_issuance_before = currency_total_issuance();

        // Set mock expectations.
        let recover_signer_ctx = MockEthereumSignatureVerifier::recover_signer_context();
        let lock_under_vesting_ctx = MockVestingInterface::lock_under_vesting_context();
        recover_signer_ctx
            .expect()
            .once()
            .with(
                predicate::eq(sig(1)),
                predicate::eq(EthereumSignatureMessageParams {
                    account_id: 42,
                    ethereum_address: eth(EthAddr::Existing),
                }),
            )
            .return_const(Some(eth(EthAddr::Existing)));
        lock_under_vesting_ctx
            .expect()
            .once()
            .with(predicate::eq(42), predicate::eq(10), predicate::always())
            .return_const(Err(DispatchError::Other("vesting interface failed")));

        // Invoke the function under test.
        assert_noop!(
            TokenClaims::claim(RuntimeOrigin::signed(42), eth(EthAddr::Existing), sig(1)),
            DispatchError::Other("vesting interface failed"),
        );

        // Assert state changes.
        assert!(<Claims<Test>>::contains_key(eth(EthAddr::Existing)));
        assert_eq!(Balances::free_balance(42), 0);
        assert_eq!(pot_account_balance_before - pot_account_balance(), 0);
        assert_eq!(
            total_claimable_balance_before - total_claimable_balance(),
            0
        );
        assert_eq!(
            currency_total_issuance_before - currency_total_issuance(),
            0
        );

        // Assert mock invocations.
        recover_signer_ctx.checkpoint();
        lock_under_vesting_ctx.checkpoint();
    });
}

/// This test verifies that when there is no claim, the claim call fails.
#[test]
fn claim_non_existing() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(!<Claims<Test>>::contains_key(eth(EthAddr::Unknown)));
        assert_eq!(Balances::free_balance(42), 0);
        let pot_account_balance_before = pot_account_balance();
        let total_claimable_balance_before = total_claimable_balance();
        let currency_total_issuance_before = currency_total_issuance();

        // Set mock expectations.
        let recover_signer_ctx = MockEthereumSignatureVerifier::recover_signer_context();
        let lock_under_vesting_ctx = MockVestingInterface::lock_under_vesting_context();
        recover_signer_ctx
            .expect()
            .once()
            .with(
                predicate::eq(sig(1)),
                predicate::eq(EthereumSignatureMessageParams {
                    account_id: 42,
                    ethereum_address: eth(EthAddr::Unknown),
                }),
            )
            .return_const(Some(eth(EthAddr::Unknown)));
        lock_under_vesting_ctx.expect().never();

        // Invoke the function under test.
        assert_noop!(
            TokenClaims::claim(RuntimeOrigin::signed(42), eth(EthAddr::Unknown), sig(1)),
            <Error<Test>>::NoClaim,
        );

        // Assert state changes.
        assert!(!<Claims<Test>>::contains_key(eth(EthAddr::Unknown)));
        assert_eq!(Balances::free_balance(42), 0);
        assert_eq!(pot_account_balance_before - pot_account_balance(), 0);
        assert_eq!(
            total_claimable_balance_before - total_claimable_balance(),
            0
        );
        assert_eq!(
            currency_total_issuance_before - currency_total_issuance(),
            0
        );

        // Assert mock invocations.
        recover_signer_ctx.checkpoint();
        lock_under_vesting_ctx.checkpoint();
    });
}

/// This test verifies that empty claims in genesis are handled correctly.
#[test]
fn genesis_empty() {
    new_test_ext_with(mock::GenesisConfig {
        balances: mock::BalancesConfig {
            balances: vec![(
                mock::Pot::account_id(),
                1, /* existential deposit only */
            )],
        },
        ..Default::default()
    })
    .execute_with_ext(|_| {
        // Check the pot balance.
        assert_eq!(pot_account_balance(), <CurrencyOf<Test>>::minimum_balance());
    });
}

/// This test verifies that the genesis builder correctly ensures the pot balance.
#[test]
#[should_panic = "invalid balance in the token claims pot account: got 124, expected 457"]
fn genesis_ensure_pot_balance_is_checked() {
    new_test_ext_with(mock::GenesisConfig {
        balances: mock::BalancesConfig {
            balances: vec![(
                mock::Pot::account_id(),
                1 /* existential deposit */ +
                123, /* total claimable amount that doesn't match the sum of claims */
            )],
        },
        token_claims: mock::TokenClaimsConfig {
            claims: vec![(
                EthereumAddress([0; 20]),
                ClaimInfo {
                    balance: 456,
                    vesting: MockVestingSchedule,
                },
            )],
            total_claimable: Some(456),
        },
        ..Default::default()
    });
}

/// This test verifies that the genesis builder asserts the equality of the configured and computed
/// total claimable balances.
#[test]
#[should_panic = "computed total claimable balance (123) is different from the one specified at the genesis config (456)"]
fn genesis_ensure_total_claimable_balance_is_asserted() {
    new_test_ext_with(mock::GenesisConfig {
        balances: mock::BalancesConfig {
            balances: vec![(
                mock::Pot::account_id(),
                1 /* existential deposit */ +
                123, /* total claimable amount */
            )],
        },
        token_claims: mock::TokenClaimsConfig {
            claims: vec![(
                EthereumAddress([0; 20]),
                ClaimInfo {
                    balance: 123, /* the only contribution to the total claimable balance */
                    vesting: MockVestingSchedule,
                },
            )],
            total_claimable: Some(456), /* the configured total claimable balance that doesn't matched the computed value */
        },
        ..Default::default()
    });
}

/// This test verifies that the genesis builder works when no assertion of the total claimable
/// balance is set.
#[test]
fn genesis_no_total_claimable_balance_assertion_works() {
    new_test_ext_with(mock::GenesisConfig {
        balances: mock::BalancesConfig {
            balances: vec![(
                mock::Pot::account_id(),
                1 /* existential deposit */ +
                123, /* total claimable amount */
            )],
        },
        token_claims: mock::TokenClaimsConfig {
            claims: vec![(
                EthereumAddress([0; 20]),
                ClaimInfo {
                    balance: 123,
                    vesting: MockVestingSchedule,
                },
            )],
            total_claimable: None, /* don't assert */
        },
        ..Default::default()
    });
}

/// This test verifies that the genesis builder does not allow conflicting keys (eth addresses)
/// in claims.
#[test]
#[should_panic = "conflicting claim found in genesis for address 0x0000000000000000000000000000000000000000"]
fn genesis_does_not_allow_same_eth_address() {
    new_test_ext_with(mock::GenesisConfig {
        balances: mock::BalancesConfig {
            balances: vec![(
                mock::Pot::account_id(),
                1 /* existential deposit */ +
                123 + 456, /* total claimable amount */
            )],
        },
        token_claims: mock::TokenClaimsConfig {
            claims: vec![
                (
                    EthereumAddress([0; 20]), /* an eth address used for the first time */
                    ClaimInfo {
                        balance: 123,
                        vesting: MockVestingSchedule,
                    },
                ),
                (
                    EthereumAddress([0; 20]), /* the same eth address used for the second time */
                    ClaimInfo {
                        balance: 456,
                        vesting: MockVestingSchedule,
                    },
                ),
            ],
            total_claimable: Some(123 + 456),
        },
        ..Default::default()
    });
}

/// This test verifies that the genesis builder allow non-conflicting keys (eth addresses)
/// in claims.
#[test]
fn genesis_allows_different_eth_address() {
    new_test_ext_with(mock::GenesisConfig {
        balances: mock::BalancesConfig {
            balances: vec![(
                mock::Pot::account_id(),
                1 /* existential deposit */ +
                123 + 456, /* total claimable amount */
            )],
        },
        token_claims: mock::TokenClaimsConfig {
            claims: vec![
                (
                    EthereumAddress([0; 20]), /* an eth address used for the first time */
                    ClaimInfo {
                        balance: 123,
                        vesting: MockVestingSchedule,
                    },
                ),
                (
                    EthereumAddress([1; 20]), /* another eth address, used for the first time */
                    ClaimInfo {
                        balance: 456,
                        vesting: MockVestingSchedule,
                    },
                ),
            ],
            total_claimable: Some(123 + 456),
        },
        ..Default::default()
    });
}

/// This test verifies that we can consume all of the claims seqentially and get to the empty
/// claimable balance in the pot but without killing the pot account.
#[test]
fn claiming_sequential() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(Balances::free_balance(42), 0);
        let pot_account_balance_before = pot_account_balance();
        let currency_total_issuance_before = currency_total_issuance();

        // Prepare the keys to iterate over all the claims.
        let claims: Vec<_> = <Claims<Test>>::iter().collect();

        // Iterate over all the claims conuming them.
        for (claim_eth_address, claim_info) in &claims {
            // Set mock expectations.
            let recover_signer_ctx = MockEthereumSignatureVerifier::recover_signer_context();
            recover_signer_ctx
                .expect()
                .once()
                .with(
                    predicate::eq(sig(1)),
                    predicate::eq(EthereumSignatureMessageParams {
                        account_id: 42,
                        ethereum_address: *claim_eth_address,
                    }),
                )
                .return_const(Some(*claim_eth_address));
            let lock_under_vesting_ctx = MockVestingInterface::lock_under_vesting_context();
            lock_under_vesting_ctx
                .expect()
                .once()
                .with(
                    predicate::eq(42),
                    predicate::eq(claim_info.balance),
                    predicate::eq(claim_info.vesting.clone()),
                )
                .return_const(Ok(()));

            assert_ok!(TokenClaims::claim(
                RuntimeOrigin::signed(42),
                *claim_eth_address,
                sig(1),
            ));

            // Assert state changes for this local iteration.
            assert!(!<Claims<Test>>::contains_key(claim_eth_address));
            assert_eq!(
                currency_total_issuance_before - currency_total_issuance(),
                0
            );

            // Assert mock invocations.
            recover_signer_ctx.checkpoint();
            lock_under_vesting_ctx.checkpoint();
        }

        // Assert overall state changes.
        assert_eq!(
            Balances::free_balance(42),
            pot_account_balance_before - <CurrencyOf<Test>>::minimum_balance()
        );
        assert_eq!(pot_account_balance(), <CurrencyOf<Test>>::minimum_balance());
        assert_eq!(total_claimable_balance(), 0);
        assert_eq!(
            currency_total_issuance_before - currency_total_issuance(),
            0
        );
    });
}

/// This test verifies that [`NoVesting`] parses from JSON correctly.
#[test]
fn parse_no_vesting_schedule() {
    let data = r#"{"balance":10,"vesting":null}"#;
    let claim_info: ClaimInfo<u8, <NoVesting<Test> as VestingInterface>::Schedule> =
        serde_json::from_str(data).unwrap();
    assert_eq!(
        claim_info,
        ClaimInfo {
            balance: 10,
            vesting: (),
        }
    )
}

/// These tests ensure that [`traits::OptionalVesting`] works as expected.
mod optional_vesting_interface {
    use super::*;
    use crate::traits::{self, OptionalVesting};

    mockall::mock! {
        #[derive(Debug)]
        pub DummyValueVestingInterface {}
        impl traits::VestingInterface for DummyValueVestingInterface {
            type AccountId = u8;
            type Balance = u8;
            type Schedule = String;

            fn lock_under_vesting(
                account: &<Self as traits::VestingInterface>::AccountId,
                balance_to_lock: <Self as traits::VestingInterface>::Balance,
                schedule: <Self as traits::VestingInterface>::Schedule,
            ) -> frame_support::dispatch::DispatchResult;
        }
    }

    type TestInterface = OptionalVesting<MockDummyValueVestingInterface>;

    /// Ensure the present value parses correctly.
    #[test]
    fn some_parses_correctly() {
        let data = r#"{"balance":10,"vesting":"test"}"#;
        let claim_info: ClaimInfo<u8, <TestInterface as VestingInterface>::Schedule> =
            serde_json::from_str(data).unwrap();
        assert_eq!(
            claim_info,
            ClaimInfo {
                balance: 10,
                vesting: Some("test".to_owned()),
            }
        );
    }

    /// Ensure the absent value parses correctly.
    #[test]
    fn none_parses_correctly() {
        let data = r#"{"balance":10,"vesting":null}"#;
        let claim_info: ClaimInfo<u8, <TestInterface as VestingInterface>::Schedule> =
            serde_json::from_str(data).unwrap();
        assert_eq!(
            claim_info,
            ClaimInfo {
                balance: 10,
                vesting: None,
            }
        );
    }

    /// Ensure that [`Some`] value properly evaluates to a call to the wrapped vesting interface and
    /// passes the [`Ok`] result as-is.
    #[test]
    fn some_works_ok() {
        mock::with_runtime_lock(|| {
            let ctx = MockDummyValueVestingInterface::lock_under_vesting_context();
            ctx.expect()
                .once()
                .with(
                    predicate::eq(42),
                    predicate::eq(10),
                    predicate::eq("test".to_owned()),
                )
                .return_const(Ok(()));

            assert_eq!(
                TestInterface::lock_under_vesting(&42, 10, Some("test".to_owned())),
                Ok(())
            );

            ctx.checkpoint();
        })
    }

    /// Ensure that [`Some`] value properly evaluates to a call to the wrapped vesting interface and
    /// passes the [`Err`] result as-is.
    #[test]
    fn some_works_err() {
        mock::with_runtime_lock(|| {
            let ctx = MockDummyValueVestingInterface::lock_under_vesting_context();
            ctx.expect()
                .once()
                .with(
                    predicate::eq(42),
                    predicate::eq(10),
                    predicate::eq("test".to_owned()),
                )
                .return_const(Err(DispatchError::Other("test error")));

            assert_eq!(
                TestInterface::lock_under_vesting(&42, 10, Some("test".to_owned())),
                Err(DispatchError::Other("test error"))
            );

            ctx.checkpoint();
        })
    }

    /// Ensure that [`None`] value does not evaluate to a call to the wrapped vesting interface at
    /// all, and simply returns [`Ok`].
    #[test]
    fn none_works() {
        mock::with_runtime_lock(|| {
            let ctx = MockDummyValueVestingInterface::lock_under_vesting_context();
            ctx.expect().never();

            assert_eq!(TestInterface::lock_under_vesting(&42, 10, None), Ok(()));

            ctx.checkpoint();
        })
    }
}

/// This test verifies that adding claim signed by sudo account works in the happy path.
#[test]
fn adding_claim_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(!<Claims<Test>>::contains_key(eth(EthAddr::New)));

        let funds_provider_balance_before = Balances::free_balance(FUNDS_PROVIDER);
        let pot_account_balance_before = pot_account_balance();
        let total_claimable_balance_before = total_claimable_balance();
        let currency_total_issuance_before = currency_total_issuance();

        let claimed_balance = 30;
        let new_claim_info = ClaimInfo {
            balance: claimed_balance,
            vesting: MockVestingSchedule,
        };

        // Set block number to enable events.
        mock::System::set_block_number(1);

        // Non-sudo accounts are not allowed.
        assert_noop!(
            TokenClaims::add_claim(
                RuntimeOrigin::signed(42),
                eth(EthAddr::New),
                new_claim_info.clone(),
                FUNDS_PROVIDER,
            ),
            DispatchError::BadOrigin
        );
        // Invoke the function under test.
        assert_ok!(TokenClaims::add_claim(
            RuntimeOrigin::root(),
            eth(EthAddr::New),
            new_claim_info.clone(),
            FUNDS_PROVIDER,
        ));

        // Assert state changes.
        assert_eq!(
            <Claims<Test>>::get(eth(EthAddr::New)).unwrap(),
            new_claim_info
        );
        assert_eq!(
            total_claimable_balance() - total_claimable_balance_before,
            claimed_balance
        );
        assert_eq!(
            pot_account_balance() - pot_account_balance_before,
            claimed_balance
        );
        assert_eq!(
            funds_provider_balance_before - Balances::free_balance(FUNDS_PROVIDER),
            claimed_balance
        );
        assert_eq!(currency_total_issuance_before, currency_total_issuance());
        mock::System::assert_has_event(mock::RuntimeEvent::TokenClaims(Event::ClaimAdded {
            ethereum_address: eth(EthAddr::New),
            claim: new_claim_info,
        }));
    });
}

/// This test verifies that adding claim signed by account different from sudo fails.
#[test]
fn adding_claim_not_sudo() {
    new_test_ext().execute_with_ext(|_| {
        let new_claim_info = ClaimInfo {
            balance: 30,
            vesting: MockVestingSchedule,
        };

        // Non-sudo accounts are not allowed.
        assert_noop!(
            TokenClaims::add_claim(
                RuntimeOrigin::signed(42),
                eth(EthAddr::New),
                new_claim_info,
                FUNDS_PROVIDER,
            ),
            DispatchError::BadOrigin
        );
    });
}

/// This test verifies that adding claim with conflicting ethereum address fails.
#[test]
fn adding_claim_conflicting_eth_address() {
    new_test_ext().execute_with_ext(|_| {
        let new_claim_info = ClaimInfo {
            balance: 30,
            vesting: MockVestingSchedule,
        };

        // Invoke the function under test.
        assert_noop!(
            TokenClaims::add_claim(
                RuntimeOrigin::root(),
                eth(EthAddr::Existing),
                new_claim_info,
                FUNDS_PROVIDER,
            ),
            Error::<Test>::ConflictingEthereumAddress
        );
    });
}

/// This test verifies that adding claim fails if there is not enough funds in the funds provider.
#[test]
fn adding_claim_funds_provider_underflow() {
    new_test_ext().execute_with_ext(|_| {
        let new_claim_info = ClaimInfo {
            balance: Balances::free_balance(FUNDS_PROVIDER) + 1,
            vesting: MockVestingSchedule,
        };

        // Invoke the function under test.
        assert_noop!(
            TokenClaims::add_claim(
                RuntimeOrigin::root(),
                eth(EthAddr::New),
                new_claim_info,
                FUNDS_PROVIDER,
            ),
            Error::<Test>::FundsProviderUnderflow
        );
    });
}

/// This test verifies that removing claim signed by sudo account works in the happy path.
#[test]
fn removing_claim_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(<Claims<Test>>::contains_key(eth(EthAddr::Existing)));

        let claim = <Claims<Test>>::get(eth(EthAddr::Existing)).unwrap();
        let funds_consumer_balance_before = Balances::free_balance(FUNDS_CONSUMER);
        let pot_account_balance_before = pot_account_balance();
        let total_claimable_balance_before = total_claimable_balance();
        let currency_total_issuance_before = currency_total_issuance();

        // Set block number to enable events.
        mock::System::set_block_number(1);

        // Invoke the function under test.
        assert_ok!(TokenClaims::remove_claim(
            RuntimeOrigin::root(),
            eth(EthAddr::Existing),
            FUNDS_CONSUMER,
        ));

        // Assert state changes.
        assert!(!<Claims<Test>>::contains_key(eth(EthAddr::Existing)));
        assert_eq!(
            total_claimable_balance_before - total_claimable_balance(),
            claim.balance
        );
        assert_eq!(
            pot_account_balance_before - pot_account_balance(),
            claim.balance
        );
        assert_eq!(
            Balances::free_balance(FUNDS_CONSUMER) - funds_consumer_balance_before,
            claim.balance
        );
        assert_eq!(currency_total_issuance_before, currency_total_issuance());
        mock::System::assert_has_event(mock::RuntimeEvent::TokenClaims(Event::ClaimRemoved {
            ethereum_address: eth(EthAddr::Existing),
            claim,
        }));
    });
}

/// This test verifies that removing claim signed by account different from sudo fails.
#[test]
fn removing_claim_not_sudo() {
    new_test_ext().execute_with_ext(|_| {
        // Non-sudo accounts are not allowed.
        assert_noop!(
            TokenClaims::remove_claim(
                RuntimeOrigin::signed(42),
                eth(EthAddr::Existing),
                FUNDS_CONSUMER,
            ),
            DispatchError::BadOrigin
        );
    });
}

/// This test verifies that removing claim fails if the claim doesn't exist.
#[test]
fn removing_claim_no_claim() {
    new_test_ext().execute_with_ext(|_| {
        // Invoke the function under test.
        assert_noop!(
            TokenClaims::remove_claim(RuntimeOrigin::root(), eth(EthAddr::New), FUNDS_CONSUMER,),
            Error::<Test>::NoClaim
        );
    });
}

/// This test verifies that changing claim with balance increase signed by sudo account works in the happy path.
#[test]
fn changing_claim_balance_increase_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(<Claims<Test>>::contains_key(eth(EthAddr::Existing)));

        let old_claim = <Claims<Test>>::get(eth(EthAddr::Existing)).unwrap();
        let funds_provider_balance_before = Balances::free_balance(FUNDS_PROVIDER);
        let pot_account_balance_before = pot_account_balance();
        let total_claimable_balance_before = total_claimable_balance();
        let currency_total_issuance_before = currency_total_issuance();

        let new_claimed_balance = 30;
        let new_claim_info = ClaimInfo {
            balance: new_claimed_balance,
            vesting: MockVestingSchedule,
        };

        // Set block number to enable events.
        mock::System::set_block_number(1);

        // Non-sudo accounts are not allowed.
        assert_noop!(
            TokenClaims::change_claim(
                RuntimeOrigin::signed(42),
                eth(EthAddr::Existing),
                new_claim_info.clone(),
                FUNDS_PROVIDER,
            ),
            DispatchError::BadOrigin
        );
        // Invoke the function under test.
        assert_ok!(TokenClaims::change_claim(
            RuntimeOrigin::root(),
            eth(EthAddr::Existing),
            new_claim_info.clone(),
            FUNDS_PROVIDER,
        ));

        // Assert state changes.
        assert_eq!(
            <Claims<Test>>::get(eth(EthAddr::Existing)).unwrap(),
            new_claim_info
        );
        assert_eq!(
            total_claimable_balance() - total_claimable_balance_before,
            new_claimed_balance - old_claim.balance
        );
        assert_eq!(
            pot_account_balance() - pot_account_balance_before,
            new_claimed_balance - old_claim.balance
        );
        assert_eq!(
            funds_provider_balance_before - Balances::free_balance(FUNDS_PROVIDER),
            new_claimed_balance - old_claim.balance
        );
        assert_eq!(currency_total_issuance_before, currency_total_issuance());
        mock::System::assert_has_event(mock::RuntimeEvent::TokenClaims(Event::ClaimChanged {
            ethereum_address: eth(EthAddr::Existing),
            old_claim,
            new_claim: new_claim_info,
        }));
    });
}

/// This test verifies that changing claim with balance decrease signed by sudo account works in the happy path.
#[test]
fn changing_claim_balance_decrease_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(<Claims<Test>>::contains_key(eth(EthAddr::Existing)));

        let old_claim = <Claims<Test>>::get(eth(EthAddr::Existing)).unwrap();
        let funds_provider_balance_before = Balances::free_balance(FUNDS_PROVIDER);
        let pot_account_balance_before = pot_account_balance();
        let total_claimable_balance_before = total_claimable_balance();
        let currency_total_issuance_before = currency_total_issuance();

        let new_claimed_balance = 5;
        let new_claim_info = ClaimInfo {
            balance: new_claimed_balance,
            vesting: MockVestingSchedule,
        };

        // Set block number to enable events.
        mock::System::set_block_number(1);

        // Non-sudo accounts are not allowed.
        assert_noop!(
            TokenClaims::change_claim(
                RuntimeOrigin::signed(42),
                eth(EthAddr::Existing),
                new_claim_info.clone(),
                FUNDS_PROVIDER,
            ),
            DispatchError::BadOrigin
        );
        // Invoke the function under test.
        assert_ok!(TokenClaims::change_claim(
            RuntimeOrigin::root(),
            eth(EthAddr::Existing),
            new_claim_info.clone(),
            FUNDS_PROVIDER,
        ));

        // Assert state changes.
        assert_eq!(
            <Claims<Test>>::get(eth(EthAddr::Existing)).unwrap(),
            new_claim_info
        );
        assert_eq!(
            total_claimable_balance_before - total_claimable_balance(),
            old_claim.balance - new_claimed_balance,
        );
        assert_eq!(
            pot_account_balance_before - pot_account_balance(),
            old_claim.balance - new_claimed_balance,
        );
        assert_eq!(
            Balances::free_balance(FUNDS_PROVIDER) - funds_provider_balance_before,
            old_claim.balance - new_claimed_balance
        );
        assert_eq!(currency_total_issuance_before, currency_total_issuance());
        mock::System::assert_has_event(mock::RuntimeEvent::TokenClaims(Event::ClaimChanged {
            ethereum_address: eth(EthAddr::Existing),
            old_claim,
            new_claim: new_claim_info,
        }));
    });
}

/// This test verifies that changing claim with balance not changing signed by sudo account works in the happy path.
#[test]
fn changing_claim_balance_not_changing_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(<Claims<Test>>::contains_key(eth(EthAddr::Existing)));

        let old_claim = <Claims<Test>>::get(eth(EthAddr::Existing)).unwrap();
        let funds_provider_balance_before = Balances::free_balance(FUNDS_PROVIDER);
        let pot_account_balance_before = pot_account_balance();
        let total_claimable_balance_before = total_claimable_balance();
        let currency_total_issuance_before = currency_total_issuance();

        let new_claimed_balance = old_claim.balance;
        let new_claim_info = ClaimInfo {
            balance: new_claimed_balance,
            vesting: MockVestingSchedule,
        };

        // Set block number to enable events.
        mock::System::set_block_number(1);

        // Non-sudo accounts are not allowed.
        assert_noop!(
            TokenClaims::change_claim(
                RuntimeOrigin::signed(42),
                eth(EthAddr::Existing),
                new_claim_info.clone(),
                FUNDS_PROVIDER,
            ),
            DispatchError::BadOrigin
        );
        // Invoke the function under test.
        assert_ok!(TokenClaims::change_claim(
            RuntimeOrigin::root(),
            eth(EthAddr::Existing),
            new_claim_info.clone(),
            FUNDS_PROVIDER,
        ));

        // Assert state changes.
        assert_eq!(
            <Claims<Test>>::get(eth(EthAddr::Existing)).unwrap(),
            new_claim_info
        );
        assert_eq!(total_claimable_balance_before, total_claimable_balance(),);
        assert_eq!(pot_account_balance_before, pot_account_balance(),);
        assert_eq!(
            Balances::free_balance(FUNDS_PROVIDER),
            funds_provider_balance_before
        );
        assert_eq!(currency_total_issuance_before, currency_total_issuance());
        mock::System::assert_has_event(mock::RuntimeEvent::TokenClaims(Event::ClaimChanged {
            ethereum_address: eth(EthAddr::Existing),
            old_claim,
            new_claim: new_claim_info,
        }));
    });
}

/// This test verifies that changing claim signed by account different from sudo fails.
#[test]
fn changing_claim_not_sudo() {
    new_test_ext().execute_with_ext(|_| {
        let new_claim_info = ClaimInfo {
            balance: 30,
            vesting: MockVestingSchedule,
        };

        // Non-sudo accounts are not allowed.
        assert_noop!(
            TokenClaims::change_claim(
                RuntimeOrigin::signed(42),
                eth(EthAddr::New),
                new_claim_info,
                FUNDS_PROVIDER,
            ),
            DispatchError::BadOrigin
        );
    });
}

/// This test verifies that changing claim fails if the claim doesn't exist.
#[test]
fn changing_claim_no_claim() {
    new_test_ext().execute_with_ext(|_| {
        let new_claim_info = ClaimInfo {
            balance: 30,
            vesting: MockVestingSchedule,
        };

        // Invoke the function under test.
        assert_noop!(
            TokenClaims::change_claim(
                RuntimeOrigin::root(),
                eth(EthAddr::New),
                new_claim_info,
                FUNDS_PROVIDER,
            ),
            Error::<Test>::NoClaim
        );
    });
}

/// This test verifies that changing claim fails if there is not enough funds in the funds provider.
#[test]
fn changing_claim_funds_provider_underflow() {
    new_test_ext().execute_with_ext(|_| {
        let current_claim_balance = <Claims<Test>>::get(eth(EthAddr::Existing)).unwrap().balance;
        let new_claim_info = ClaimInfo {
            balance: Balances::free_balance(FUNDS_PROVIDER) + current_claim_balance + 1,
            vesting: MockVestingSchedule,
        };

        // Invoke the function under test.
        assert_noop!(
            TokenClaims::change_claim(
                RuntimeOrigin::root(),
                eth(EthAddr::Existing),
                new_claim_info,
                FUNDS_PROVIDER,
            ),
            Error::<Test>::FundsProviderUnderflow
        );
    });
}

/// This test verifies that signed extension's `validate` works in the happy path.
#[test]
fn signed_ext_validate_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(<Claims<Test>>::contains_key(eth(EthAddr::Existing)));
        assert_eq!(Balances::free_balance(42), 0);

        // Set mock expectations.
        let recover_signer_ctx = MockEthereumSignatureVerifier::recover_signer_context();
        recover_signer_ctx
            .expect()
            .once()
            .with(
                predicate::eq(sig(1)),
                predicate::eq(EthereumSignatureMessageParams {
                    account_id: 42,
                    ethereum_address: eth(EthAddr::Existing),
                }),
            )
            .return_const(Some(eth(EthAddr::Existing)));
        let lock_under_vesting_ctx = MockVestingInterface::lock_under_vesting_context();
        lock_under_vesting_ctx.expect().never();

        // Invoke the function under test.
        let normal = DispatchInfo {
            weight: Weight::from_ref_time(100),
            class: DispatchClass::Normal,
            pays_fee: Pays::No,
        };
        let len = 0;
        let ext = <CheckTokenClaim<Test>>::new();
        assert_storage_noop!(assert_ok!(
            ext.validate(
                &42,
                &mock::RuntimeCall::TokenClaims(Call::claim {
                    ethereum_address: eth(EthAddr::Existing),
                    ethereum_signature: sig(1),
                }),
                &normal,
                len
            ),
            ValidTransaction::default()
        ));

        // Assert mock invocations.
        recover_signer_ctx.checkpoint();
        lock_under_vesting_ctx.checkpoint();
    });
}

/// This test verifies that signed extension's `validate` properly fails when the eth signature is
/// invalid.
#[test]
fn signed_ext_validate_fails_invalid_eth_signature() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(<Claims<Test>>::contains_key(eth(EthAddr::Existing)));
        assert_eq!(Balances::free_balance(42), 0);

        // Set mock expectations.
        let recover_signer_ctx = MockEthereumSignatureVerifier::recover_signer_context();
        recover_signer_ctx
            .expect()
            .once()
            .with(
                predicate::eq(sig(1)),
                predicate::eq(EthereumSignatureMessageParams {
                    account_id: 42,
                    ethereum_address: eth(EthAddr::Existing),
                }),
            )
            .return_const(None);
        let lock_under_vesting_ctx = MockVestingInterface::lock_under_vesting_context();
        lock_under_vesting_ctx.expect().never();

        // Invoke the function under test.
        let normal = DispatchInfo {
            weight: Weight::from_ref_time(100),
            class: DispatchClass::Normal,
            pays_fee: Pays::No,
        };
        let len = 0;
        let ext = <CheckTokenClaim<Test>>::new();
        assert_noop!(
            ext.validate(
                &42,
                &mock::RuntimeCall::TokenClaims(Call::claim {
                    ethereum_address: eth(EthAddr::Existing),
                    ethereum_signature: sig(1),
                }),
                &normal,
                len
            ),
            TransactionValidityError::Invalid(InvalidTransaction::BadProof)
        );

        // Assert mock invocations.
        recover_signer_ctx.checkpoint();
        lock_under_vesting_ctx.checkpoint();
    });
}

/// This test verifies that signed extension's `validate` properly fails when the claim is
/// not present in the state for the requested eth address.
#[test]
fn signed_ext_validate_fails_when_claim_is_absent() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(!<Claims<Test>>::contains_key(eth(EthAddr::Unknown)));
        assert_eq!(Balances::free_balance(42), 0);

        // Set mock expectations.
        let recover_signer_ctx = MockEthereumSignatureVerifier::recover_signer_context();
        recover_signer_ctx
            .expect()
            .once()
            .with(
                predicate::eq(sig(1)),
                predicate::eq(EthereumSignatureMessageParams {
                    account_id: 42,
                    ethereum_address: eth(EthAddr::Unknown),
                }),
            )
            .return_const(Some(eth(EthAddr::Unknown)));
        let lock_under_vesting_ctx = MockVestingInterface::lock_under_vesting_context();
        lock_under_vesting_ctx.expect().never();

        // Invoke the function under test.
        let normal = DispatchInfo {
            weight: Weight::from_ref_time(100),
            class: DispatchClass::Normal,
            pays_fee: Pays::No,
        };
        let len = 0;
        let ext = <CheckTokenClaim<Test>>::new();
        assert_noop!(
            ext.validate(
                &42,
                &mock::RuntimeCall::TokenClaims(Call::claim {
                    ethereum_address: eth(EthAddr::Unknown),
                    ethereum_signature: sig(1),
                }),
                &normal,
                len
            ),
            TransactionValidityError::Invalid(InvalidTransaction::Call)
        );

        // Assert mock invocations.
        recover_signer_ctx.checkpoint();
        lock_under_vesting_ctx.checkpoint();
    });
}
