use frame_support::sp_runtime::{ArithmeticError, DispatchError};

use crate::{genesis_verifier::Balanced, mock::*};

/// This test verifies that the genesis builder correctly ensures genesis verifier result in case
/// it returns false.
#[test]
#[should_panic = "invalid genesis bridge pot currency swap related data"]
fn genesis_verifier_false() {
    with_runtime_lock(|| {
        let verify_ctx = MockGenesisVerifier::verify_context();
        verify_ctx.expect().once().return_const(false);

        let config = GenesisConfig {
            swap_bridge_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Unchecked,
            },
            ..Default::default()
        };

        new_test_ext_with(config).execute_with(move || {});

        verify_ctx.checkpoint();
    })
}

/// This test verifies that the genesis builder correctly ensures genesis verifier result in case
/// it returns true.
#[test]
fn genesis_verifier_true() {
    with_runtime_lock(|| {
        let verify_ctx = MockGenesisVerifier::verify_context();
        verify_ctx.expect().once().return_const(true);

        let config = GenesisConfig {
            swap_bridge_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Unchecked,
            },
            ..Default::default()
        };

        new_test_ext_with(config).execute_with(move || {});

        verify_ctx.checkpoint();
    })
}

/// This test verifies that `calculate_expected_to_bridge_balance` function works in the happy path.
#[test]
fn calculate_expected_to_bridge_balance_works() {
    with_runtime_lock(|| {
        let all_from_balances_without_bridge_balance = vec![10, 20, 30, 40];
        let expected_to_bridge_balance =
            all_from_balances_without_bridge_balance.iter().sum::<u64>() + EXISTENTIAL_DEPOSIT;
        assert_eq!(
            expected_to_bridge_balance,
            Balanced::<SwapBridge>::calculate_expected_to_bridge_balance(
                all_from_balances_without_bridge_balance
            )
            .unwrap()
        );
    })
}

/// This test verifies that `calculate_expected_to_bridge_balance` function fails in case
/// overflow error happens.
#[test]
fn calculate_expected_to_bridge_balance_fails_overflow() {
    with_runtime_lock(|| {
        let all_from_balances_without_bridge_balance = vec![10, 20, 30, u64::MAX];
        assert_eq!(
            DispatchError::Arithmetic(ArithmeticError::Overflow),
            Balanced::<SwapBridge>::calculate_expected_to_bridge_balance(
                all_from_balances_without_bridge_balance
            )
            .unwrap_err()
        );
    })
}
