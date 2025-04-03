//! Tests regarding the functionality of the fungible trait set implementations.

use frame_support::{
    assert_noop, assert_ok,
    traits::{
        fungible::{Balanced, Inspect, Mutate, Unbalanced},
        tokens::Precision,
    },
};
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
fn active_issuance_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check the active issuance value.
        assert_eq!(EvmBalances::active_issuance(), 2 * INIT_BALANCE);
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
fn total_balance_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check the total balance value.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);
    });
}

#[test]
fn balance_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check the balance value.
        assert_eq!(EvmBalances::balance(&ALICE), INIT_BALANCE);
    });
}

#[test]
fn reducable_balance_works() {
    new_test_ext().execute_with_ext(|_| {
        assert_eq!(
            EvmBalances::reducible_balance(&ALICE, Preservation::Expendable, Fortitude::Polite),
            INIT_BALANCE
        );

        assert_eq!(
            EvmBalances::reducible_balance(&ALICE, Preservation::Expendable, Fortitude::Force),
            INIT_BALANCE
        );

        assert_eq!(
            EvmBalances::reducible_balance(&ALICE, Preservation::Preserve, Fortitude::Polite),
            INIT_BALANCE - 1
        );

        assert_eq!(
            EvmBalances::reducible_balance(&ALICE, Preservation::Preserve, Fortitude::Force),
            INIT_BALANCE - 1
        );

        assert_eq!(
            EvmBalances::reducible_balance(&ALICE, Preservation::Protect, Fortitude::Polite),
            INIT_BALANCE - 1
        );

        assert_eq!(
            EvmBalances::reducible_balance(&ALICE, Preservation::Protect, Fortitude::Force),
            INIT_BALANCE - 1
        );
    });
}

#[test]
fn can_deposit_works_success() {
    new_test_ext().execute_with_ext(|_| {
        assert_eq!(
            EvmBalances::can_deposit(&ALICE, 10, Provenance::Minted),
            DepositConsequence::Success
        );
    });
}

#[test]
fn can_deposit_works_overflow() {
    new_test_ext().execute_with_ext(|_| {
        assert_eq!(
            EvmBalances::can_deposit(&ALICE, u64::MAX, Provenance::Minted),
            DepositConsequence::Overflow
        );
    });
}

#[test]
fn can_withdraw_works_success() {
    new_test_ext().execute_with_ext(|_| {
        assert_eq!(
            EvmBalances::can_withdraw(&ALICE, 10),
            WithdrawConsequence::Success
        );
    });
}

#[test]
fn can_withdraw_works_underflow() {
    new_test_ext().execute_with_ext(|_| {
        assert_eq!(
            EvmBalances::can_withdraw(&ALICE, u64::MAX),
            WithdrawConsequence::Underflow
        );
    });
}

#[test]
fn can_withdraw_works_balance_low() {
    new_test_ext().execute_with_ext(|_| {
        assert_eq!(
            EvmBalances::can_withdraw(&ALICE, INIT_BALANCE + 1),
            WithdrawConsequence::BalanceLow
        );
    });
}

#[test]
fn can_withdraw_works_reduced_to_zero() {
    new_test_ext().execute_with_ext(|_| {
        assert_eq!(
            EvmBalances::can_withdraw(&ALICE, INIT_BALANCE),
            WithdrawConsequence::ReducedToZero(0)
        );
    });
}

#[test]
fn write_balance_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);

        let write_balance = 10;

        // Invoke the function under test.
        assert_eq!(EvmBalances::write_balance(&ALICE, write_balance), Ok(None));

        // Assert state changes.
        assert_eq!(EvmBalances::total_balance(&ALICE), write_balance);
    });
}

#[test]
fn set_total_issuance_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        let set_total_issuance_balance = 100;

        // Invoke the function under test.
        EvmBalances::set_total_issuance(set_total_issuance_balance);

        // Assert state changes.
        assert_eq!(EvmBalances::total_issuance(), set_total_issuance_balance);
    });
}

#[test]
fn decrease_balance_works_exact_expendable() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);

        let decreased_balance = 100;

        // Invoke the function under test.
        assert_ok!(EvmBalances::decrease_balance(
            &ALICE,
            decreased_balance,
            Precision::Exact,
            Preservation::Expendable,
            Fortitude::Polite
        ));

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&ALICE),
            INIT_BALANCE - decreased_balance
        );
    });
}

#[test]
fn decrease_balance_works_best_effort_preserve() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);

        let decreased_balance = INIT_BALANCE + 1;

        // Invoke the function under test.
        assert_ok!(EvmBalances::decrease_balance(
            &ALICE,
            decreased_balance,
            Precision::BestEffort,
            Preservation::Preserve,
            Fortitude::Polite
        ));

        // Assert state changes.
        assert_eq!(EvmBalances::total_balance(&ALICE), 1);
    });
}

#[test]
fn decrease_balance_works_full_balance() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);

        // Set block number to enable events.
        System::set_block_number(1);

        let decreased_balance = INIT_BALANCE;

        // Invoke the function under test.
        assert_ok!(EvmBalances::decrease_balance(
            &ALICE,
            decreased_balance,
            Precision::Exact,
            Preservation::Expendable,
            Fortitude::Polite
        ));

        // Assert state changes.
        assert_eq!(EvmBalances::total_balance(&ALICE), 0);
        assert!(!EvmSystem::account_exists(&ALICE));
        System::assert_has_event(RuntimeEvent::EvmSystem(
            pallet_evm_system::Event::KilledAccount { account: ALICE },
        ));
    });
}

#[test]
fn decrease_balance_fails_funds_unavailable() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);

        let decreased_balance = INIT_BALANCE + 1;

        // Invoke the function under test.
        assert_noop!(
            EvmBalances::decrease_balance(
                &ALICE,
                decreased_balance,
                Precision::Exact,
                Preservation::Preserve,
                Fortitude::Polite
            ),
            TokenError::FundsUnavailable
        );
    });
}

#[test]
fn increase_balance_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);

        let increased_balance = 100;

        // Invoke the function under test.
        assert_ok!(EvmBalances::increase_balance(
            &ALICE,
            increased_balance,
            Precision::Exact,
        ));

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&ALICE),
            INIT_BALANCE + increased_balance
        );
    });
}

#[test]
fn increase_balance_works_best_effort() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);

        let increased_balance = u64::MAX;

        // Invoke the function under test.
        assert_ok!(EvmBalances::increase_balance(
            &ALICE,
            increased_balance,
            Precision::BestEffort,
        ));

        // Assert state changes.
        assert_eq!(EvmBalances::total_balance(&ALICE), u64::MAX);
    });
}

#[test]
fn increase_balance_fails_overflow() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);

        let increased_balance = u64::MAX;

        // Invoke the function under test.
        assert_noop!(
            EvmBalances::increase_balance(&ALICE, increased_balance, Precision::Exact),
            ArithmeticError::Overflow
        );
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
fn mint_into_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        // Set block number to enable events.
        System::set_block_number(1);

        let minted_balance = 10;

        // Invoke the function under test.
        assert_ok!(EvmBalances::mint_into(&ALICE, minted_balance));

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&ALICE),
            INIT_BALANCE + minted_balance
        );
        assert_eq!(
            EvmBalances::total_issuance(),
            2 * INIT_BALANCE + minted_balance
        );
        System::assert_has_event(RuntimeEvent::EvmBalances(Event::Minted {
            who: ALICE,
            amount: minted_balance,
        }));

        assert_total_issuance_invariant();
    });
}

#[test]
fn mint_into_fails_overflow() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);

        let minted_balance = u64::MAX;

        // Invoke the function under test.
        assert_noop!(
            EvmBalances::mint_into(&ALICE, minted_balance),
            ArithmeticError::Overflow
        );
    });
}

#[test]
fn burn_from_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        // Set block number to enable events.
        System::set_block_number(1);

        let burned_balance = 10;

        // Invoke the function under test.
        assert_ok!(EvmBalances::burn_from(
            &ALICE,
            burned_balance,
            Precision::Exact,
            Fortitude::Polite
        ));

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&ALICE),
            INIT_BALANCE - burned_balance
        );
        assert_eq!(
            EvmBalances::total_issuance(),
            2 * INIT_BALANCE - burned_balance
        );
        System::assert_has_event(RuntimeEvent::EvmBalances(Event::Burned {
            who: ALICE,
            amount: burned_balance,
        }));

        assert_total_issuance_invariant();
    });
}

#[test]
fn burn_from_works_best_effort() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        // Set block number to enable events.
        System::set_block_number(1);

        let burned_balance = INIT_BALANCE + 1;

        // Invoke the function under test.
        assert_ok!(EvmBalances::burn_from(
            &ALICE,
            burned_balance,
            Precision::BestEffort,
            Fortitude::Polite
        ));

        // Assert state changes.
        assert_eq!(EvmBalances::total_balance(&ALICE), 0);
        assert_eq!(EvmBalances::total_issuance(), INIT_BALANCE);
        System::assert_has_event(RuntimeEvent::EvmBalances(Event::Burned {
            who: ALICE,
            amount: INIT_BALANCE,
        }));
        assert!(!EvmSystem::account_exists(&ALICE));
        System::assert_has_event(RuntimeEvent::EvmSystem(
            pallet_evm_system::Event::KilledAccount { account: ALICE },
        ));

        assert_total_issuance_invariant();
    });
}

#[test]
fn burn_from_works_exact_full_balance() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        // Set block number to enable events.
        System::set_block_number(1);

        let burned_balance = INIT_BALANCE;

        // Invoke the function under test.
        assert_ok!(EvmBalances::burn_from(
            &ALICE,
            burned_balance,
            Precision::Exact,
            Fortitude::Polite
        ));

        // Assert state changes.
        assert_eq!(EvmBalances::total_balance(&ALICE), 0);
        assert_eq!(EvmBalances::total_issuance(), INIT_BALANCE);
        System::assert_has_event(RuntimeEvent::EvmBalances(Event::Burned {
            who: ALICE,
            amount: INIT_BALANCE,
        }));
        assert!(!EvmSystem::account_exists(&ALICE));
        System::assert_has_event(RuntimeEvent::EvmSystem(
            pallet_evm_system::Event::KilledAccount { account: ALICE },
        ));

        assert_total_issuance_invariant();
    });
}

#[test]
fn burn_from_fails_funds_unavailable() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        // Set block number to enable events.
        System::set_block_number(1);

        let burned_balance = INIT_BALANCE + 1;

        // Invoke the function under test.
        assert_noop!(
            EvmBalances::burn_from(&ALICE, burned_balance, Precision::Exact, Fortitude::Polite),
            TokenError::FundsUnavailable
        );
    });
}

#[test]
fn shelve_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        // Set block number to enable events.
        System::set_block_number(1);

        let shelved_balance = 10;

        // Invoke the function under test.
        assert_ok!(EvmBalances::shelve(&ALICE, shelved_balance));

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&ALICE),
            INIT_BALANCE - shelved_balance
        );
        assert_eq!(
            EvmBalances::total_issuance(),
            2 * INIT_BALANCE - shelved_balance
        );
        System::assert_has_event(RuntimeEvent::EvmBalances(Event::Suspended {
            who: ALICE,
            amount: shelved_balance,
        }));

        assert_total_issuance_invariant();
    });
}

#[test]
fn shelve_works_exact_full_balance() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        // Set block number to enable events.
        System::set_block_number(1);

        let shelved_balance = INIT_BALANCE;

        // Invoke the function under test.
        assert_ok!(EvmBalances::shelve(&ALICE, shelved_balance));

        // Assert state changes.
        assert_eq!(EvmBalances::total_balance(&ALICE), 0);
        assert_eq!(EvmBalances::total_issuance(), INIT_BALANCE);
        System::assert_has_event(RuntimeEvent::EvmBalances(Event::Suspended {
            who: ALICE,
            amount: INIT_BALANCE,
        }));
        assert!(!EvmSystem::account_exists(&ALICE));
        System::assert_has_event(RuntimeEvent::EvmSystem(
            pallet_evm_system::Event::KilledAccount { account: ALICE },
        ));

        assert_total_issuance_invariant();
    });
}

#[test]
fn shelve_fails_funds_unavailable() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        // Set block number to enable events.
        System::set_block_number(1);

        let shelved_balance = INIT_BALANCE + 1;

        // Invoke the function under test.
        assert_noop!(
            EvmBalances::shelve(&ALICE, shelved_balance),
            TokenError::FundsUnavailable
        );
    });
}

#[test]
fn restore_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        // Set block number to enable events.
        System::set_block_number(1);

        let restored_balance = 10;

        // Invoke the function under test.
        assert_ok!(EvmBalances::restore(&ALICE, restored_balance));

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&ALICE),
            INIT_BALANCE + restored_balance
        );
        assert_eq!(
            EvmBalances::total_issuance(),
            2 * INIT_BALANCE + restored_balance
        );
        System::assert_has_event(RuntimeEvent::EvmBalances(Event::Restored {
            who: ALICE,
            amount: restored_balance,
        }));

        assert_total_issuance_invariant();
    });
}

#[test]
fn restore_fails_overflow() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);

        let restored_balance = u64::MAX;

        // Invoke the function under test.
        assert_noop!(
            EvmBalances::restore(&ALICE, restored_balance),
            ArithmeticError::Overflow
        );
    });
}

#[test]
fn transfer_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);

        let transferred_amount = 100;

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_ok!(EvmBalances::transfer(
            &ALICE,
            &BOB,
            transferred_amount,
            Preservation::Preserve
        ));

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&ALICE),
            INIT_BALANCE - transferred_amount
        );
        assert_eq!(
            EvmBalances::total_balance(&BOB),
            INIT_BALANCE + transferred_amount
        );
        System::assert_has_event(RuntimeEvent::EvmBalances(Event::Transfer {
            from: ALICE,
            to: BOB,
            amount: transferred_amount,
        }));

        assert_total_issuance_invariant();
    });
}

#[test]
fn transfer_works_full_balance() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);

        let transferred_amount = INIT_BALANCE;

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_ok!(EvmBalances::transfer(
            &ALICE,
            &BOB,
            transferred_amount,
            Preservation::Expendable
        ));

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&ALICE),
            INIT_BALANCE - transferred_amount
        );
        assert_eq!(
            EvmBalances::total_balance(&BOB),
            INIT_BALANCE + transferred_amount
        );
        System::assert_has_event(RuntimeEvent::EvmBalances(Event::Transfer {
            from: ALICE,
            to: BOB,
            amount: transferred_amount,
        }));
        assert!(!EvmSystem::account_exists(&ALICE));
        System::assert_has_event(RuntimeEvent::EvmSystem(
            pallet_evm_system::Event::KilledAccount { account: ALICE },
        ));

        assert_total_issuance_invariant();
    });
}

#[test]
fn transfer_fails_funds_unavailable() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);

        let transferred_amount = INIT_BALANCE + 1;

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_noop!(
            EvmBalances::transfer(&ALICE, &BOB, transferred_amount, Preservation::Preserve),
            TokenError::FundsUnavailable
        );
    });
}

#[test]
fn transfer_fails_not_expendable() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);

        let transferred_amount = INIT_BALANCE;

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        assert_noop!(
            EvmBalances::transfer(&ALICE, &BOB, transferred_amount, Preservation::Preserve),
            TokenError::NotExpendable
        );
    });
}

#[test]
fn transfer_fails_underflow() {
    new_test_ext().execute_with_ext(|_| {
        // Prepare test preconditions.
        let charlie = 3;
        let eve = 4;
        EvmBalances::set_balance(&charlie, u64::MAX);
        EvmBalances::set_balance(&eve, 1);

        // Invoke the function under test.
        assert_noop!(
            EvmBalances::transfer(&charlie, &eve, u64::MAX, Preservation::Expendable),
            // Withdraw consequence is checked first by reducing total issuance.
            ArithmeticError::Underflow,
        );
    });
}

#[test]
fn rescind_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        // Set block number to enable events.
        System::set_block_number(1);

        let rescinded_balance = 100;

        // Burn some balance.
        let imbalance = EvmBalances::rescind(rescinded_balance);

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_issuance(),
            2 * INIT_BALANCE - rescinded_balance
        );
        drop(imbalance);
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);
        System::assert_has_event(RuntimeEvent::EvmBalances(Event::Rescinded {
            amount: rescinded_balance,
        }));

        assert_total_issuance_invariant();
    });
}

#[test]
fn issue_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        // Set block number to enable events.
        System::set_block_number(1);

        let issued_balance = 100;

        // Burn some balance.
        let imbalance = EvmBalances::issue(issued_balance);

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_issuance(),
            2 * INIT_BALANCE + issued_balance
        );
        drop(imbalance);
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);
        System::assert_has_event(RuntimeEvent::EvmBalances(Event::Issued {
            amount: issued_balance,
        }));

        assert_total_issuance_invariant();
    });
}

#[test]
fn deposit_flow_works() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);

        let deposited_amount = 10;

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        let debt = EvmBalances::deposit(&ALICE, deposited_amount, Precision::Exact).unwrap();

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&ALICE),
            INIT_BALANCE + deposited_amount
        );
        System::assert_has_event(RuntimeEvent::EvmBalances(Event::Deposit {
            who: ALICE,
            amount: deposited_amount,
        }));
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        let _ = EvmBalances::settle(&BOB, debt, Preservation::Expendable);
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        assert_total_issuance_invariant();
    });
}

#[test]
fn deposit_works_new_account() {
    new_test_ext().execute_with_ext(|_| {
        let charlie = 3;

        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&charlie), 0);

        let deposited_amount = 10;

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        let debt = EvmBalances::deposit(&charlie, deposited_amount, Precision::Exact).unwrap();

        // Assert state changes.
        assert_eq!(EvmBalances::total_balance(&charlie), deposited_amount);
        System::assert_has_event(RuntimeEvent::EvmBalances(Event::Deposit {
            who: charlie,
            amount: deposited_amount,
        }));
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);
        let _ = EvmBalances::settle(&BOB, debt, Preservation::Expendable);
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);
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
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);

        let withdrawed_amount = 1000;

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        let credit = EvmBalances::withdraw(
            &ALICE,
            withdrawed_amount,
            Precision::Exact,
            Preservation::Preserve,
            Fortitude::Polite,
        )
        .unwrap();

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&ALICE),
            INIT_BALANCE - withdrawed_amount
        );
        System::assert_has_event(RuntimeEvent::EvmBalances(Event::Withdraw {
            who: ALICE,
            amount: withdrawed_amount,
        }));
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);
        let _ = EvmBalances::resolve(&BOB, credit);
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        assert_total_issuance_invariant();
    });
}

#[test]
fn withdraw_works_full_balance() {
    new_test_ext().execute_with_ext(|_| {
        // Check test preconditions.
        assert_eq!(EvmBalances::total_balance(&ALICE), INIT_BALANCE);

        let withdrawed_amount = INIT_BALANCE;

        // Set block number to enable events.
        System::set_block_number(1);

        // Invoke the function under test.
        let credit = EvmBalances::withdraw(
            &ALICE,
            withdrawed_amount,
            Precision::Exact,
            Preservation::Expendable,
            Fortitude::Polite,
        )
        .unwrap();

        // Assert state changes.
        assert_eq!(
            EvmBalances::total_balance(&ALICE),
            INIT_BALANCE - withdrawed_amount
        );
        System::assert_has_event(RuntimeEvent::EvmBalances(crate::Event::Withdraw {
            who: ALICE,
            amount: withdrawed_amount,
        }));
        assert!(!EvmSystem::account_exists(&ALICE));
        System::assert_has_event(RuntimeEvent::EvmSystem(
            pallet_evm_system::Event::KilledAccount { account: ALICE },
        ));
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);
        let _ = EvmBalances::resolve(&BOB, credit);
        assert_eq!(EvmBalances::total_issuance(), 2 * INIT_BALANCE);

        assert_total_issuance_invariant();
    });
}
