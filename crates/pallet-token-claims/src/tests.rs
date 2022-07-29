//! The tests for the pallet.

use frame_support::{assert_noop, assert_ok};
use mockall::predicate;
use primitives_ethereum::EthereumAddress;
use sp_runtime::DispatchError;

use crate::{
    mock::{
        eth, new_test_ext, sig, Balances, MockEthereumSignatureVerifier, MockVestingInterface,
        MockVestingSchedule, Origin, Test, TestExternalitiesExt, TokenClaims,
    },
    types::{ClaimInfo, EthereumSignatureMessageParams},
    *,
};

#[test]
fn basic_setup_works() {
    new_test_ext().execute_with_ext(|_| {
        assert_eq!(<Claims<Test>>::get(&EthereumAddress::default()), None);
        assert_eq!(
            <Claims<Test>>::get(&eth(1)),
            Some(ClaimInfo {
                balance: 10,
                vesting: None
            })
        );
        assert_eq!(
            <Claims<Test>>::get(&eth(2)),
            Some(ClaimInfo {
                balance: 20,
                vesting: Some(MockVestingSchedule)
            })
        );
    });
}

/// This test verifies that claiming works in the happy path (when there is no vesting).
#[test]
fn claiming_works_no_vesting() {
    new_test_ext().execute_with_ext(|_| {
        assert!(<Claims<Test>>::contains_key(&eth(1)));
        assert_eq!(Balances::free_balance(42), 0);

        let recover_signer_ctx = MockEthereumSignatureVerifier::recover_signer_context();
        recover_signer_ctx
            .expect()
            .once()
            .returning(|_, _| Some(eth(1)));
        let lock_under_vesting_ctx = MockVestingInterface::lock_under_vesting_context();
        lock_under_vesting_ctx.expect().never();

        assert_ok!(TokenClaims::claim(Origin::signed(42), eth(1), sig(1),));

        assert!(!<Claims<Test>>::contains_key(&eth(1)));
        assert_eq!(Balances::free_balance(42), 10);

        recover_signer_ctx.checkpoint();
        lock_under_vesting_ctx.checkpoint();
    });
}

/// This test verifies that claiming works in the happy path with vesting.
#[test]
fn claiming_works_with_vesting() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(<Claims<Test>>::contains_key(&eth(1)));
        assert_eq!(Balances::free_balance(42), 0);

        // Set mock expectations.
        let recover_signer_ctx = MockEthereumSignatureVerifier::recover_signer_context();
        let lock_under_vesting_ctx = MockVestingInterface::lock_under_vesting_context();
        recover_signer_ctx
            .expect()
            .once()
            .with(
                predicate::eq(EthereumSignatureMessageParams {
                    account_id: 42,
                    ethereum_address: eth(2),
                }),
                predicate::eq(sig(1)),
            )
            .return_const(Some(eth(2)));
        lock_under_vesting_ctx
            .expect()
            .once()
            .with(predicate::eq(42), predicate::eq(20), predicate::always())
            .return_const(Ok(()));

        // Invoke the function under test.
        assert_ok!(TokenClaims::claim(Origin::signed(42), eth(2), sig(1)));

        // Assert state changes.
        assert!(!<Claims<Test>>::contains_key(&eth(2)));
        assert_eq!(Balances::free_balance(42), 20);

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
        assert!(<Claims<Test>>::contains_key(&eth(2)));
        assert_eq!(Balances::free_balance(42), 0);

        // Set mock expectations.
        let recover_signer_ctx = MockEthereumSignatureVerifier::recover_signer_context();
        let lock_under_vesting_ctx = MockVestingInterface::lock_under_vesting_context();
        recover_signer_ctx
            .expect()
            .once()
            .with(
                predicate::eq(EthereumSignatureMessageParams {
                    account_id: 42,
                    ethereum_address: eth(1),
                }),
                predicate::eq(sig(1)),
            )
            .return_const(None);
        lock_under_vesting_ctx.expect().never();

        // Invoke the function under test.
        assert_noop!(
            TokenClaims::claim(Origin::signed(42), eth(1), sig(1)),
            <Error<Test>>::InvalidSignature
        );

        // Assert state changes.
        assert!(<Claims<Test>>::contains_key(&eth(1)));
        assert_eq!(Balances::free_balance(42), 0);

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
        assert!(<Claims<Test>>::contains_key(&eth(2)));
        assert_eq!(Balances::free_balance(42), 0);

        // Set mock expectations.
        let recover_signer_ctx = MockEthereumSignatureVerifier::recover_signer_context();
        let lock_under_vesting_ctx = MockVestingInterface::lock_under_vesting_context();
        recover_signer_ctx
            .expect()
            .once()
            .with(
                predicate::eq(EthereumSignatureMessageParams {
                    account_id: 42,
                    ethereum_address: eth(1),
                }),
                predicate::eq(sig(1)),
            )
            .return_const(Some(eth(2)));
        lock_under_vesting_ctx.expect().never();

        // Invoke the function under test.
        assert_noop!(
            TokenClaims::claim(Origin::signed(42), eth(1), sig(1)),
            <Error<Test>>::InvalidSignature
        );

        // Assert state changes.
        assert!(<Claims<Test>>::contains_key(&eth(1)));
        assert_eq!(Balances::free_balance(42), 0);

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
        assert!(<Claims<Test>>::contains_key(&eth(2)));
        assert_eq!(Balances::free_balance(42), 0);

        // Set mock expectations.
        let recover_signer_ctx = MockEthereumSignatureVerifier::recover_signer_context();
        let lock_under_vesting_ctx = MockVestingInterface::lock_under_vesting_context();
        recover_signer_ctx
            .expect()
            .once()
            .with(
                predicate::eq(EthereumSignatureMessageParams {
                    account_id: 42,
                    ethereum_address: eth(2),
                }),
                predicate::eq(sig(1)),
            )
            .return_const(Some(eth(2)));
        lock_under_vesting_ctx
            .expect()
            .once()
            .with(predicate::eq(42), predicate::eq(20), predicate::always())
            .return_const(Err(DispatchError::Other("vesting interface failed")));

        // Invoke the function under test.
        assert_noop!(
            TokenClaims::claim(Origin::signed(42), eth(2), sig(1)),
            DispatchError::Other("vesting interface failed"),
        );

        // Assert state changes.
        assert!(<Claims<Test>>::contains_key(&eth(2)));
        assert_eq!(Balances::free_balance(42), 0);

        // Assert mock invocations.
        recover_signer_ctx.checkpoint();
        lock_under_vesting_ctx.checkpoint();
    });
}
