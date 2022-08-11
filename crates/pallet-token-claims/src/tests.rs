//! The tests for the pallet.

use frame_support::{assert_noop, assert_ok};
use mockall::predicate;
use primitives_ethereum::EthereumAddress;
use sp_runtime::DispatchError;

use crate::{
    mock::{
        eth, new_test_ext, new_test_ext_with, sig, Balances, EthAddr,
        MockEthereumSignatureVerifier, MockVestingInterface, MockVestingSchedule, Origin, Test,
        TestExternalitiesExt, TokenClaims,
    },
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
        assert_eq!(<Claims<Test>>::get(&EthereumAddress::default()), None);
        assert_eq!(
            <Claims<Test>>::get(&eth(EthAddr::NoVesting)),
            Some(ClaimInfo {
                balance: 10,
                vesting: None
            })
        );
        assert_eq!(
            <Claims<Test>>::get(&eth(EthAddr::WithVesting)),
            Some(ClaimInfo {
                balance: 20,
                vesting: Some(MockVestingSchedule)
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

/// This test verifies that claiming works in the happy path (when there is no vesting).
#[test]
fn claiming_works_no_vesting() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(<Claims<Test>>::contains_key(&eth(EthAddr::NoVesting)));
        assert_eq!(Balances::free_balance(42), 0);
        let pot_account_balance_before = pot_account_balance();
        let total_claimable_balance_before = total_claimable_balance();
        let currency_total_issuance_before = currency_total_issuance();

        // Set mock expectations.
        let recover_signer_ctx = MockEthereumSignatureVerifier::recover_signer_context();
        recover_signer_ctx
            .expect()
            .once()
            .with(
                predicate::eq(sig(1)),
                predicate::eq(EthereumSignatureMessageParams {
                    account_id: 42,
                    ethereum_address: eth(EthAddr::NoVesting),
                }),
            )
            .return_const(Some(eth(EthAddr::NoVesting)));
        let lock_under_vesting_ctx = MockVestingInterface::lock_under_vesting_context();
        lock_under_vesting_ctx.expect().never();

        // Invoke the function under test.
        assert_ok!(TokenClaims::claim(
            Origin::signed(42),
            eth(EthAddr::NoVesting),
            sig(1),
        ));

        // Assert state changes.
        assert!(!<Claims<Test>>::contains_key(&eth(EthAddr::NoVesting)));
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

/// This test verifies that claiming works in the happy path with vesting.
#[test]
fn claiming_works_with_vesting() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(<Claims<Test>>::contains_key(&eth(EthAddr::WithVesting)));
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
                    ethereum_address: eth(EthAddr::WithVesting),
                }),
            )
            .return_const(Some(eth(EthAddr::WithVesting)));
        lock_under_vesting_ctx
            .expect()
            .once()
            .with(predicate::eq(42), predicate::eq(20), predicate::always())
            .return_const(Ok(()));

        // Invoke the function under test.
        assert_ok!(TokenClaims::claim(
            Origin::signed(42),
            eth(EthAddr::WithVesting),
            sig(1)
        ));

        // Assert state changes.
        assert!(!<Claims<Test>>::contains_key(&eth(EthAddr::WithVesting)));
        assert_eq!(Balances::free_balance(42), 20);
        assert_eq!(pot_account_balance_before - pot_account_balance(), 20);
        assert_eq!(
            total_claimable_balance_before - total_claimable_balance(),
            20
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
        assert!(<Claims<Test>>::contains_key(&eth(EthAddr::NoVesting)));
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
                    ethereum_address: eth(EthAddr::NoVesting),
                }),
            )
            .return_const(None);
        lock_under_vesting_ctx.expect().never();

        // Invoke the function under test.
        assert_noop!(
            TokenClaims::claim(Origin::signed(42), eth(EthAddr::NoVesting), sig(1)),
            <Error<Test>>::InvalidSignature
        );

        // Assert state changes.
        assert!(<Claims<Test>>::contains_key(&eth(EthAddr::NoVesting)));
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
        assert!(<Claims<Test>>::contains_key(&eth(EthAddr::NoVesting)));
        assert!(!<Claims<Test>>::contains_key(&eth(EthAddr::Unknown)));
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
                    ethereum_address: eth(EthAddr::NoVesting),
                }),
            )
            .return_const(Some(eth(EthAddr::Unknown)));
        lock_under_vesting_ctx.expect().never();

        // Invoke the function under test.
        assert_noop!(
            TokenClaims::claim(Origin::signed(42), eth(EthAddr::NoVesting), sig(1)),
            <Error<Test>>::InvalidSignature
        );

        // Assert state changes.
        assert!(<Claims<Test>>::contains_key(&eth(EthAddr::NoVesting)));
        assert!(!<Claims<Test>>::contains_key(&eth(EthAddr::Unknown)));
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
        assert!(<Claims<Test>>::contains_key(&eth(EthAddr::WithVesting)));
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
                    ethereum_address: eth(EthAddr::WithVesting),
                }),
            )
            .return_const(Some(eth(EthAddr::WithVesting)));
        lock_under_vesting_ctx
            .expect()
            .once()
            .with(predicate::eq(42), predicate::eq(20), predicate::always())
            .return_const(Err(DispatchError::Other("vesting interface failed")));

        // Invoke the function under test.
        assert_noop!(
            TokenClaims::claim(Origin::signed(42), eth(EthAddr::WithVesting), sig(1)),
            DispatchError::Other("vesting interface failed"),
        );

        // Assert state changes.
        assert!(<Claims<Test>>::contains_key(&eth(EthAddr::WithVesting)));
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
        assert!(!<Claims<Test>>::contains_key(&eth(EthAddr::Unknown)));
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
            TokenClaims::claim(Origin::signed(42), eth(EthAddr::Unknown), sig(1)),
            <Error<Test>>::NoClaim,
        );

        // Assert state changes.
        assert!(!<Claims<Test>>::contains_key(&eth(EthAddr::Unknown)));
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
                    vesting: None,
                },
            )],
            total_claimable: Some(456),
        },
        ..Default::default()
    });
}

/// This test verifies that the genesis builder asserted the equality of the configured and computed
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
                    vesting: None,
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
                    vesting: None,
                },
            )],
            total_claimable: None, /* don't assert */
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

            match claim_info.vesting {
                Some(ref vesting) => lock_under_vesting_ctx
                    .expect()
                    .once()
                    .with(
                        predicate::eq(42),
                        predicate::eq(claim_info.balance),
                        predicate::eq(vesting.clone()),
                    )
                    .return_const(Ok(())),
                None => lock_under_vesting_ctx.expect().never(),
            };

            assert_ok!(TokenClaims::claim(
                Origin::signed(42),
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
