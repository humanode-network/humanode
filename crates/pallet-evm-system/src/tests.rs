//! The tests for the pallet.

use frame_support::{assert_noop, assert_storage_noop};
use mockall::predicate;
use sp_core::H160;
use sp_std::str::FromStr;

use crate::{mock::*, *};

/// This test verifies that creating an account works as expected
/// in case a new account should be created.
#[test]
fn create_account_created() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test data.
        let account_id = H160::from_str("1000000000000000000000000000000000000001").unwrap();

        // Check test preconditions.
        assert!(!EvmSystem::account_exists(&account_id));

        // Set block number to enable events.
        System::set_block_number(1);

        // Set mock expectations.
        let on_new_account_ctx = MockDummyOnNewAccount::on_new_account_context();
        on_new_account_ctx
            .expect()
            .once()
            .with(predicate::eq(account_id))
            .return_const(());

        // Invoke the function under test.
        assert_eq!(
            EvmSystem::create_account(&account_id),
            AccountCreationOutcome::Created
        );

        // Assert state changes.
        assert!(EvmSystem::account_exists(&account_id));
        assert_eq!(
            <Account<Test>>::get(account_id),
            AccountInfo::<_, _>::default()
        );
        System::assert_has_event(RuntimeEvent::EvmSystem(Event::NewAccount {
            account: account_id,
        }));

        // Assert mock invocations.
        on_new_account_ctx.checkpoint();
    });
}

/// This test verifies that creating an account works as expected
/// when the account already exists.
#[test]
fn create_account_already_exists() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test data.
        let account_id = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        <Account<Test>>::insert(account_id, AccountInfo::<_, _>::default());

        // Invoke the function under test.
        assert_storage_noop!(assert_eq!(
            EvmSystem::create_account(&account_id),
            AccountCreationOutcome::AlreadyExists
        ));
    });
}

/// This test verifies that incrementing account nonce works in the happy path
/// in case a new account should be created.
#[test]
fn inc_account_nonce_account_created() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test data.
        let account_id = H160::from_str("1000000000000000000000000000000000000001").unwrap();

        // Check test preconditions.
        assert!(!EvmSystem::account_exists(&account_id));

        let nonce_before = EvmSystem::account_nonce(&account_id);

        // Set block number to enable events.
        System::set_block_number(1);

        // Set mock expectations.
        let on_new_account_ctx = MockDummyOnNewAccount::on_new_account_context();
        on_new_account_ctx
            .expect()
            .once()
            .with(predicate::eq(account_id))
            .return_const(());

        // Invoke the function under test.
        EvmSystem::inc_account_nonce(&account_id);

        // Assert state changes.
        assert_eq!(EvmSystem::account_nonce(&account_id), nonce_before + 1);
        assert!(EvmSystem::account_exists(&account_id));
        System::assert_has_event(RuntimeEvent::EvmSystem(Event::NewAccount {
            account: account_id,
        }));

        // Invoke the function under test again to check that the account is not being created now.
        EvmSystem::inc_account_nonce(&account_id);
        // Assert state changes.
        assert_eq!(EvmSystem::account_nonce(&account_id), nonce_before + 2);
        assert!(EvmSystem::account_exists(&account_id));

        // Assert mock invocations.
        on_new_account_ctx.checkpoint();
    });
}

/// This test verifies that incrementing account nonce works in the happy path
/// in case an account already exists.
#[test]
fn inc_account_nonce_account_exists() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test data.
        let account_id = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        <Account<Test>>::insert(account_id, AccountInfo::<_, _>::default());

        // Check test preconditions.
        assert!(EvmSystem::account_exists(&account_id));

        let nonce_before = EvmSystem::account_nonce(&account_id);

        // Invoke the function under test.
        EvmSystem::inc_account_nonce(&account_id);

        // Assert state changes.
        assert!(EvmSystem::account_exists(&account_id));
        assert_eq!(EvmSystem::account_nonce(&account_id), nonce_before + 1);
    });
}

/// This test verifies that `try_mutate_exists` works as expected in case data wasn't providing
/// and returned data is `Some`. As a result, a new account has been created.
#[test]
fn try_mutate_exists_account_created() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test data.
        let account_id = H160::from_str("1000000000000000000000000000000000000001").unwrap();

        // Check test preconditions.
        assert!(!EvmSystem::account_exists(&account_id));

        // Set mock expectations.
        let on_new_account_ctx = MockDummyOnNewAccount::on_new_account_context();
        on_new_account_ctx
            .expect()
            .once()
            .with(predicate::eq(account_id))
            .return_const(());

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        EvmSystem::try_mutate_exists(&account_id, |maybe_data| -> Result<(), DispatchError> {
            *maybe_data = Some(1);
            Ok(())
        })
        .unwrap();

        // Assert state changes.
        assert!(EvmSystem::account_exists(&account_id));
        assert_eq!(
            <Account<Test>>::get(account_id),
            AccountInfo {
                data: 1,
                ..Default::default()
            }
        );
        System::assert_has_event(RuntimeEvent::EvmSystem(Event::NewAccount {
            account: account_id,
        }));

        // Assert mock invocations.
        on_new_account_ctx.checkpoint();
    });
}

/// This test verifies that `try_mutate_exists` works as expected in case data was providing
/// and returned data is `Some`. As a result, the account has been updated.
#[test]
fn try_mutate_exists_account_updated() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test data.
        let account_id = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let nonce = 1;
        let data = 1;
        <Account<Test>>::insert(account_id, AccountInfo { nonce, data });

        // Check test preconditions.
        assert!(EvmSystem::account_exists(&account_id));

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        EvmSystem::try_mutate_exists(&account_id, |maybe_data| -> Result<(), DispatchError> {
            if let Some(ref mut data) = maybe_data {
                *data += 1;
            }
            Ok(())
        })
        .unwrap();

        // Assert state changes.
        assert!(EvmSystem::account_exists(&account_id));
        assert_eq!(
            <Account<Test>>::get(account_id),
            AccountInfo {
                nonce,
                data: data + 1,
            }
        );
    });
}

/// This test verifies that `try_mutate_exists` works as expected in case data was providing
/// and returned data is `None`, account has zero nonce. As a result, the account has been removed.
#[test]
fn try_mutate_exists_account_removed() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test data.
        let account_id = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        <Account<Test>>::insert(account_id, AccountInfo::<_, _>::default());

        // Check test preconditions.
        assert!(EvmSystem::account_exists(&account_id));

        // Set mock expectations.
        let is_precompile_ctx = MockIsPrecompile::is_precompile_context();
        is_precompile_ctx
            .expect()
            .once()
            .with(predicate::eq(account_id))
            .return_const(false);

        let on_killed_account_ctx = MockDummyOnKilledAccount::on_killed_account_context();
        on_killed_account_ctx
            .expect()
            .once()
            .with(predicate::eq(account_id))
            .return_const(());

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        EvmSystem::try_mutate_exists(&account_id, |maybe_data| -> Result<(), DispatchError> {
            *maybe_data = None;
            Ok(())
        })
        .unwrap();

        // Assert state changes.
        assert!(!EvmSystem::account_exists(&account_id));
        System::assert_has_event(RuntimeEvent::EvmSystem(Event::KilledAccount {
            account: account_id,
        }));

        // Assert mock invocations.
        is_precompile_ctx.checkpoint();
        on_killed_account_ctx.checkpoint();
    });
}

/// This test verifies that `try_mutate_exists` works as expected in case data was providing
/// and returned data is `None`, account has non zero nonce. As a result, the account has been retained.
#[test]
fn try_mutate_exists_account_retained() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test data.
        let account_id = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let nonce = 10;
        let data = 100;

        let account_info = AccountInfo { nonce, data };
        <Account<Test>>::insert(account_id, account_info);

        // Check test preconditions.
        assert!(EvmSystem::account_exists(&account_id));

        // Invoke the function under test.
        EvmSystem::try_mutate_exists(&account_id, |maybe_data| -> Result<(), DispatchError> {
            *maybe_data = None;
            Ok(())
        })
        .unwrap();

        // Assert state changes.
        assert!(EvmSystem::account_exists(&account_id));
        assert_eq!(
            <Account<Test>>::get(account_id),
            AccountInfo {
                nonce,
                ..Default::default()
            }
        );
    });
}

/// This test verifies that `try_mutate_exists` works as expected in case data was providing
/// and returned data is `None`, account is precompiled. As a result, the account has been retained.
#[test]
fn try_mutate_exists_precompiled_account_retained() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test data.
        let precompile = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let nonce = 0;
        let data = 100;

        let account_info = AccountInfo { nonce, data };
        <Account<Test>>::insert(precompile, account_info);

        // Check test preconditions.
        assert!(EvmSystem::account_exists(&precompile));

        // Set mock expectations.
        let is_precompile_ctx = MockIsPrecompile::is_precompile_context();
        is_precompile_ctx
            .expect()
            .once()
            .with(predicate::eq(precompile))
            .return_const(true);

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        EvmSystem::try_mutate_exists(&precompile, |maybe_data| -> Result<(), DispatchError> {
            *maybe_data = None;
            Ok(())
        })
        .unwrap();

        // Assert state changes.
        assert!(EvmSystem::account_exists(&precompile));
        assert_eq!(<Account<Test>>::get(precompile), AccountInfo::default());

        // Assert that there is no a corresponding `KilledAccount` event.
        assert!(System::events().iter().all(|record| record.event
            != RuntimeEvent::EvmSystem(Event::KilledAccount {
                account: precompile,
            })));

        // Assert mock invocations.
        is_precompile_ctx.checkpoint();
    });
}

/// This test verifies that `try_mutate_exists` works as expected in case data wasn't providing
/// and returned data is `None`. As a result, the account hasn't been created.
#[test]
fn try_mutate_exists_account_not_created() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test data.
        let account_id = H160::from_str("1000000000000000000000000000000000000001").unwrap();

        // Check test preconditions.
        assert!(!EvmSystem::account_exists(&account_id));

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_storage_noop!(<Account<Test>>::try_mutate_exists(
            account_id,
            |maybe_data| -> Result<(), ()> {
                *maybe_data = None;
                Ok(())
            }
        )
        .unwrap());
    });
}

/// This test verifies that `try_mutate_exists` works as expected in case getting error
/// during data mutation.
#[test]
fn try_mutate_exists_without_changes() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test data.
        let account_id = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        <Account<Test>>::insert(account_id, AccountInfo::<_, _>::default());

        // Check test preconditions.
        assert!(EvmSystem::account_exists(&account_id));

        // Invoke the function under test.
        assert_noop!(
            <Account<Test>>::try_mutate_exists(account_id, |maybe_data| -> Result<(), ()> {
                *maybe_data = None;
                Err(())
            }),
            ()
        );
    });
}
