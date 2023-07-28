use frame_support::sp_runtime::ArithmeticError;

use crate::{
    genesis_verifier::{Balanced, Error},
    mock::*,
};

/// This test verifies that the genesis builder correctly ensures genesis verifier result in case
/// it returns false.
#[test]
#[should_panic = "invalid genesis bridge pot currency swap related data"]
fn genesis_verifier_false() {
    with_runtime_lock(|| {
        let verify_lr_ctx = MockGenesisVerifierLR::verify_context();
        let verify_rl_ctx = MockGenesisVerifierLR::verify_context();
        verify_lr_ctx.expect().once().return_const(false);
        // Swap bridge left genesis config presents early than swap bridge right genesis config
        // at mocked runtime.
        verify_rl_ctx.expect().never();

        let config = GenesisConfig {
            swap_bridge_left_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Unchecked,
            },
            swap_bridge_right_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Unchecked,
            },
            ..Default::default()
        };

        new_test_ext_with(config).execute_with(move || {});

        verify_lr_ctx.checkpoint();
        verify_rl_ctx.checkpoint();
    })
}

/// This test verifies that the genesis builder correctly ensures genesis verifier result in case
/// it returns true.
#[test]
fn genesis_verifier_true() {
    with_runtime_lock(|| {
        let verify_lr_ctx = MockGenesisVerifierLR::verify_context();
        let verify_rl_ctx = MockGenesisVerifierRL::verify_context();
        verify_lr_ctx.expect().once().return_const(true);
        verify_rl_ctx.expect().once().return_const(true);

        let config = GenesisConfig {
            swap_bridge_left_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Unchecked,
            },
            swap_bridge_right_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Unchecked,
            },
            ..Default::default()
        };

        new_test_ext_with(config).execute_with(move || {});

        verify_lr_ctx.checkpoint();
        verify_rl_ctx.checkpoint();
    })
}

/// This test verifies that `calculate_expected_bridge_balance` function works in the happy path.
#[test]
fn calculate_expected_bridge_balance_works() {
    with_runtime_lock(|| {
        let from_balances = vec![10, 20, 30, 40];
        let expected_to_bridge_balance = from_balances.iter().sum::<u64>() + EXISTENTIAL_DEPOSIT;
        assert_eq!(
            expected_to_bridge_balance,
            Balanced::<SwapBridgeLeft>::calculate_expected_bridge_balance(from_balances).unwrap()
        );
    })
}

/// This test verifies that `calculate_expected_bridge_balance` function fails in case
/// overflow error happens.
#[test]
fn calculate_expected_bridge_balance_fails_overflow() {
    with_runtime_lock(|| {
        let from_balances = vec![10, 20, 30, u64::MAX];
        assert_eq!(
            Error::Arithmetic(ArithmeticError::Overflow),
            Balanced::<SwapBridgeLeft>::calculate_expected_bridge_balance(from_balances)
                .unwrap_err()
        );
    })
}
