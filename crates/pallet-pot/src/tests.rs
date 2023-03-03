//! The tests for the pallet.

use frame_support::{assert_noop, assert_ok};
use mockall::predicate;
use primitives_ethereum::EthereumAddress;

use crate::{
    mock::{
        eth, new_test_ext, sig, EthAddr, EvmAccountsMapping, MockSignedClaimVerifier,
        RuntimeOrigin, Test, TestExternalitiesExt,
    },
    *,
};

/// This test verifies that basic genesis setup works in the happy path.
#[test]
fn basic_setup_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check the evm accounts mapping.
        assert_eq!(<Accounts<Test>>::get(EthereumAddress::default()), None);
        assert_eq!(<Accounts<Test>>::get(eth(EthAddr::Existing)), Some(42));
        assert_eq!(<EthereumAddresses<Test>>::get(10), None);
        assert_eq!(
            <EthereumAddresses<Test>>::get(42),
            Some(eth(EthAddr::Existing))
        );
    });
}

/// This test verifies that claiming account works in the happy path.
#[test]
fn claiming_account_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(<Accounts<Test>>::get(eth(EthAddr::New)), None);
        assert_eq!(<EthereumAddresses<Test>>::get(10), None);

        // Set mock expectations.
        let signed_claim_verifier_ctx = MockSignedClaimVerifier::verify_context();
        signed_claim_verifier_ctx
            .expect()
            .once()
            .with(predicate::eq(10), predicate::eq(sig(1)))
            .return_const(Some(eth(EthAddr::New)));

        // Set block number to enable events.
        mock::System::set_block_number(1);

        // Invoke the function under test.
        assert_ok!(EvmAccountsMapping::claim_account(
            RuntimeOrigin::signed(10),
            eth(EthAddr::New),
            sig(1)
        ));

        // Assert state changes.
        assert_eq!(<Accounts<Test>>::get(eth(EthAddr::New)), Some(10));
        assert_eq!(<EthereumAddresses<Test>>::get(10), Some(eth(EthAddr::New)));
        mock::System::assert_has_event(mock::RuntimeEvent::EvmAccountsMapping(
            Event::ClaimAccount {
                account_id: 10,
                ethereum_address: eth(EthAddr::New),
            },
        ));

        // Assert mock invocations.
        signed_claim_verifier_ctx.checkpoint();
    });
}

/// This test verifies that claiming account does not go through when the submitted native address already mapped.
#[test]
fn claim_account_native_address_already_mapped() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(<EthereumAddresses<Test>>::get(42).is_some());

        // Set mock expectations.
        let signed_claim_verifier_ctx = MockSignedClaimVerifier::verify_context();
        signed_claim_verifier_ctx.expect().never();

        // Invoke the function under test.
        assert_noop!(
            EvmAccountsMapping::claim_account(RuntimeOrigin::signed(42), eth(EthAddr::New), sig(1)),
            <Error<Test>>::NativeAddressAlreadyMapped
        );

        // Assert mock invocations.
        signed_claim_verifier_ctx.checkpoint();
    });
}

/// This test verifies that claiming account does not go through when the submitted ethereum address already mapped.
#[test]
fn claim_account_ethereum_address_already_mapped() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(<Accounts<Test>>::get(eth(EthAddr::Existing)).is_some());

        // Set mock expectations.
        let signed_claim_verifier_ctx = MockSignedClaimVerifier::verify_context();
        signed_claim_verifier_ctx.expect().never();

        // Invoke the function under test.
        assert_noop!(
            EvmAccountsMapping::claim_account(
                RuntimeOrigin::signed(10),
                eth(EthAddr::Existing),
                sig(1)
            ),
            <Error<Test>>::EthereumAddressAlreadyMapped
        );

        // Assert mock invocations.
        signed_claim_verifier_ctx.checkpoint();
    });
}

/// This test verifies that claiming does not go through when the ethereum address recovery from
/// the ethereum signature fails.
#[test]
fn claim_account_bad_ethereum_signature() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(<Accounts<Test>>::get(eth(EthAddr::Existing)), Some(42));
        assert_eq!(
            <EthereumAddresses<Test>>::get(42),
            Some(eth(EthAddr::Existing))
        );

        // Set mock expectations.
        let signed_claim_verifier_ctx = MockSignedClaimVerifier::verify_context();
        signed_claim_verifier_ctx
            .expect()
            .once()
            .with(predicate::eq(10), predicate::eq(sig(1)))
            .return_const(None);

        // Invoke the function under test.
        assert_noop!(
            EvmAccountsMapping::claim_account(RuntimeOrigin::signed(10), eth(EthAddr::New), sig(1)),
            <Error<Test>>::BadEthereumSignature
        );

        // Assert mock invocations.
        signed_claim_verifier_ctx.checkpoint();
    });
}

/// This test verifies that claiming account does not go through when the ethereum address recovery from
/// the ethereum signature recoves an address that does not match the expected one.
#[test]
fn claim_account_invalid_ethereum_signature() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(<Accounts<Test>>::get(eth(EthAddr::Existing)), Some(42));
        assert_eq!(
            <EthereumAddresses<Test>>::get(42),
            Some(eth(EthAddr::Existing))
        );

        // Set mock expectations.
        let signed_claim_verifier_ctx = MockSignedClaimVerifier::verify_context();
        signed_claim_verifier_ctx
            .expect()
            .once()
            .with(predicate::eq(10), predicate::eq(sig(1)))
            .return_const(Some(eth(EthAddr::Unknown)));

        // Invoke the function under test.
        assert_noop!(
            EvmAccountsMapping::claim_account(RuntimeOrigin::signed(10), eth(EthAddr::New), sig(1)),
            <Error<Test>>::InvalidEthereumSignature
        );

        // Assert mock invocations.
        signed_claim_verifier_ctx.checkpoint();
    });
}
