//! Tests regarding the functionality of the `Currency` trait set implementations.

use frame_support::{assert_noop, assert_ok, traits::Currency};
use sp_runtime::TokenError;

use crate::{mock::*, tests::assert_total_issuance_invariant, *};

#[test]
fn total_issuance_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check the total issuance value.
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);
    });
}

#[test]
fn total_balance_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check the total balance value.
        assert_eq!(EvmBalances::total_balance(&alice()), INIT_BALANCE);
    });
}

#[test]
fn free_balance_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check the free balance value.
        assert_eq!(EvmBalances::free_balance(alice()), INIT_BALANCE);
    });
}

#[test]
fn can_slash_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check possible slashing if slashing value is less than current balance.
        assert!(EvmBalances::can_slash(&alice(), 100));

        // Check possible slashing if slashing value is equal to current balance.
        assert!(EvmBalances::can_slash(&alice(), INIT_BALANCE));

        // Check slashing restriction if slashing value that is greater than current balance.
        assert!(!EvmBalances::can_slash(&alice(), INIT_BALANCE + 1));
    });
}

#[test]
fn active_issuance_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check the active issuance value.
        assert_eq!(EvmBalances::active_issuance(), 2 * INIT_BALANCE);
    });
}

#[test]
fn deactivate_reactivate_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(<InactiveIssuance<Test>>::get(), 0);

        // Deactivate some balance.
        EvmBalances::deactivate(100);
        // Assert state changes.
        assert_eq!(<InactiveIssuance<Test>>::get(), 100);

        // Reactivate some balance.
        EvmBalances::reactivate(40);
        // Assert state changes.
        assert_eq!(<InactiveIssuance<Test>>::get(), 60);

        assert_total_issuance_invariant();
    });
}

#[test]
fn minimum_balance_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check the minimum balance value.
        assert_eq!(EvmBalances::minimum_balance(), 1);
    });
}

#[test]
fn burn_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        // Burn some balance.
        let imbalance = EvmBalances::burn(100);

        // Assert state changes.
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE - 100);
        drop(imbalance);
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        assert_total_issuance_invariant();
    });
}

#[test]
fn issue_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        // Issue some balance.
        let imbalance = EvmBalances::issue(100);

        // Assert state changes.
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE + 100);
        drop(imbalance);
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        assert_total_issuance_invariant();
    });
}

#[test]
fn transfer_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice()), INIT_BALANCE);

        let transferred_amount = 100;

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_ok!(EvmBalances::transfer(
            &alice(),
            &bob(),
            transferred_amount,
            ExistenceRequirement::KeepAlive
        ));

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&alice()),
            INIT_BALANCE - transferred_amount
        );
        assert_eq!(
            EvmBalances::total_balance(&bob()),
            INIT_BALANCE + transferred_amount
        );
        System::assert_has_event(RuntimeEvent::EvmBalances(crate::Event::Transfer {
            from: alice(),
            to: bob(),
            amount: transferred_amount,
        }));

        assert_total_issuance_invariant();
    });
}

#[test]
fn transfer_works_full_balance() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice()), INIT_BALANCE);

        let transferred_amount = INIT_BALANCE;

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_ok!(EvmBalances::transfer(
            &alice(),
            &bob(),
            transferred_amount,
            ExistenceRequirement::AllowDeath
        ));

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&alice()),
            INIT_BALANCE - transferred_amount
        );
        assert_eq!(
            EvmBalances::total_balance(&bob()),
            INIT_BALANCE + transferred_amount
        );
        System::assert_has_event(RuntimeEvent::EvmBalances(crate::Event::Transfer {
            from: alice(),
            to: bob(),
            amount: transferred_amount,
        }));
        assert!(!EvmSystem::account_exists(&alice()));
        System::assert_has_event(RuntimeEvent::EvmSystem(
            pallet_evm_system::Event::KilledAccount { account: alice() },
        ));

        assert_total_issuance_invariant();
    });
}

#[test]
fn transfer_fails_funds_unavailable() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice()), INIT_BALANCE);

        let transferred_amount = INIT_BALANCE + 1;

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_noop!(
            EvmBalances::transfer(
                &alice(),
                &bob(),
                transferred_amount,
                ExistenceRequirement::KeepAlive
            ),
            TokenError::FundsUnavailable
        );
    });
}

#[test]
fn transfer_fails_not_expendable() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice()), INIT_BALANCE);

        let transferred_amount = INIT_BALANCE;

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_noop!(
            EvmBalances::transfer(
                &alice(),
                &bob(),
                transferred_amount,
                ExistenceRequirement::KeepAlive
            ),
            TokenError::NotExpendable
        );
    });
}

#[test]
fn slash_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice()), INIT_BALANCE);

        let slashed_amount = 1000;

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert!(EvmBalances::slash(&alice(), slashed_amount).1.is_zero());

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&alice()),
            INIT_BALANCE - slashed_amount
        );
        System::assert_has_event(RuntimeEvent::EvmBalances(crate::Event::Slashed {
            who: alice(),
            amount: slashed_amount,
        }));

        assert_total_issuance_invariant();
    });
}

#[test]
fn slash_works_full_balance() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice()), INIT_BALANCE);

        let slashed_amount = INIT_BALANCE;

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert!(EvmBalances::slash(&alice(), slashed_amount).1.is_zero());

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&alice()),
            INIT_BALANCE - slashed_amount
        );
        System::assert_has_event(RuntimeEvent::EvmBalances(crate::Event::Slashed {
            who: alice(),
            amount: slashed_amount,
        }));
        assert!(!EvmSystem::account_exists(&alice()));
        System::assert_has_event(RuntimeEvent::EvmSystem(
            pallet_evm_system::Event::KilledAccount { account: alice() },
        ));

        assert_total_issuance_invariant();
    });
}

#[test]
fn deposit_into_existing_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice()), INIT_BALANCE);

        let deposited_amount = 10;

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        let imbalance = EvmBalances::deposit_into_existing(&alice(), deposited_amount).unwrap();
        drop(imbalance);

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&alice()),
            INIT_BALANCE + deposited_amount
        );
        System::assert_has_event(RuntimeEvent::EvmBalances(crate::Event::Deposit {
            who: alice(),
            amount: deposited_amount,
        }));

        assert_total_issuance_invariant();
    });
}

#[test]
fn deposit_into_existing_fails_overflow() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice()), INIT_BALANCE);

        let deposited_amount = u64::MAX;

        // Invoke the function under test.
        assert_noop!(
            EvmBalances::deposit_into_existing(&alice(), deposited_amount),
            ArithmeticError::Overflow
        );
    });
}

#[test]
fn deposit_into_existing_fails_dead_account() {
    new_test_ext().execute_with_ext(|_| {
        let charlie = 3;

        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&charlie), 0);

        let deposited_amount = 10;

        // Invoke the function under test.
        assert_noop!(
            EvmBalances::deposit_into_existing(&charlie, deposited_amount),
            Error::<Test>::DeadAccount
        );
    });
}

#[test]
fn deposit_creating_works() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test preconditions.
        let charlie = 3;
        let deposited_amount = 10;
        assert!(!EvmSystem::account_exists(&charlie));

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        let _ = EvmBalances::deposit_creating(&charlie, deposited_amount);

        // Assert state changes.
        assert_eq!(EvmBalances::total_balance(&charlie), deposited_amount);
        System::assert_has_event(RuntimeEvent::EvmBalances(crate::Event::Deposit {
            who: charlie,
            amount: deposited_amount,
        }));
        assert!(EvmSystem::account_exists(&charlie));
        System::assert_has_event(RuntimeEvent::EvmSystem(
            pallet_evm_system::Event::NewAccount { account: charlie },
        ));

        assert_total_issuance_invariant();
    });
}

#[test]
fn withdraw_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice()), INIT_BALANCE);

        let withdrawed_amount = 1000;

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        let imbalance = EvmBalances::withdraw(
            &alice(),
            withdrawed_amount,
            WithdrawReasons::FEE,
            ExistenceRequirement::KeepAlive,
        )
        .unwrap();
        drop(imbalance);

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&alice()),
            INIT_BALANCE - withdrawed_amount
        );
        System::assert_has_event(RuntimeEvent::EvmBalances(crate::Event::Withdraw {
            who: alice(),
            amount: withdrawed_amount,
        }));

        assert_total_issuance_invariant();
    });
}

#[test]
fn withdraw_works_full_balance() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice()), INIT_BALANCE);

        let withdrawed_amount = INIT_BALANCE;

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        let imbalance = EvmBalances::withdraw(
            &alice(),
            withdrawed_amount,
            WithdrawReasons::FEE,
            ExistenceRequirement::AllowDeath,
        )
        .unwrap();
        drop(imbalance);

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&alice()),
            INIT_BALANCE - withdrawed_amount
        );
        System::assert_has_event(RuntimeEvent::EvmBalances(crate::Event::Withdraw {
            who: alice(),
            amount: withdrawed_amount,
        }));
        assert!(!EvmSystem::account_exists(&alice()));
        System::assert_has_event(RuntimeEvent::EvmSystem(
            pallet_evm_system::Event::KilledAccount { account: alice() },
        ));

        assert_total_issuance_invariant();
    });
}

#[test]
fn withdraw_fails_insufficient_balance() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice()), INIT_BALANCE);

        let withdrawed_amount = INIT_BALANCE + 1;

        // Invoke the function under test.
        assert_noop!(
            EvmBalances::withdraw(
                &alice(),
                withdrawed_amount,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::AllowDeath
            ),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn withdraw_fails_expendability() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&alice()), INIT_BALANCE);

        let withdrawed_amount = INIT_BALANCE;

        // Invoke the function under test.
        assert_noop!(
            EvmBalances::withdraw(
                &alice(),
                withdrawed_amount,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::KeepAlive
            ),
            Error::<Test>::Expendability
        );
    });
}

#[test]
fn make_free_balance_be_works() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test preconditions.
        let charlie = 3;
        let made_free_balance = 100;

        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&charlie), 0);

        // Invoke the function under test.
        let _ = EvmBalances::make_free_balance_be(&charlie, made_free_balance);

        // Assert state changes.
        assert_eq!(EvmBalances::total_balance(&charlie), made_free_balance);

        assert_total_issuance_invariant();
    });
}

#[test]
fn evm_system_account_should_be_reaped() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert!(EvmSystem::account_exists(&bob()));

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_ok!(EvmBalances::transfer(
            &bob(),
            &alice(),
            INIT_BALANCE,
            ExistenceRequirement::AllowDeath
        ));

        // Assert state changes.
        assert_eq!(EvmBalances::free_balance(bob()), 0);
        assert!(!EvmSystem::account_exists(&bob()));
        System::assert_has_event(RuntimeEvent::EvmSystem(
            pallet_evm_system::Event::KilledAccount { account: bob() },
        ));

        assert_total_issuance_invariant();
    });
}

#[test]
fn transferring_too_high_value_should_not_panic() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test preconditions.
        let charlie = 3;
        let eve = 4;
        EvmBalances::make_free_balance_be(&charlie, u64::MAX);
        EvmBalances::make_free_balance_be(&eve, 1);

        // Invoke the function under test.
        assert_noop!(
            EvmBalances::transfer(&charlie, &eve, u64::MAX, ExistenceRequirement::AllowDeath),
            ArithmeticError::Overflow,
        );
    });
}
