use frame_support::sp_runtime::ArithmeticError;

use crate::mock::*;
use crate::{Balanced, BalancedError};

/// This test verifies that the genesis builder correctly ensures genesis bridge pot balances
/// values that are balanced.
#[test]
fn genesis_balanced_true() {
    with_runtime_lock(|| {
        let config = GenesisConfig {
            balances_left: pallet_balances::GenesisConfig {
                balances: vec![
                    (4201, 10),
                    (4202, 20),
                    (4203, 30),
                    (4204, 40),
                    (SwapBridgeLeftPot::account_id(), 310),
                ],
            },
            balances_right: pallet_balances::GenesisConfig {
                balances: vec![
                    (4211, 20),
                    (4212, 40),
                    (4213, 60),
                    (4214, 80),
                    (4215, 100),
                    (SwapBridgeRightPot::account_id(), 120),
                ],
            },
            swap_bridge_left_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Unchecked,
            },
            swap_bridge_right_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Unchecked,
            },
            ..Default::default()
        };

        new_test_ext_with(config).execute_with(move || {});
    })
}

/// This test verifies that the genesis builder panics in case bridge pot balances values
/// are not balanced.
#[test]
#[should_panic = "genesis bridge pot balances values not balanced"]
fn genesis_balanced_false() {
    with_runtime_lock(|| {
        let config = GenesisConfig {
            balances_left: pallet_balances::GenesisConfig {
                balances: vec![(4201, 10), (SwapBridgeLeftPot::account_id(), 20)],
            },
            balances_right: pallet_balances::GenesisConfig {
                balances: vec![(4211, 20), (SwapBridgeRightPot::account_id(), 100)],
            },
            swap_bridge_left_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Initialized,
            },
            swap_bridge_right_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Initialized,
            },
            ..Default::default()
        };

        new_test_ext_with(config).execute_with(move || {});
    })
}

/// This test verifies that the genesis builder panics in case error happens during bridge pot
/// balance calculation.
#[test]
#[should_panic = "error during bridge pot balance calculation: An underflow would occur"]
fn genesis_bridge_pot_calculation_fails() {
    with_runtime_lock(|| {
        let config = GenesisConfig {
            balances_left: pallet_balances::GenesisConfig {
                balances: vec![
                    (4201, 10),
                    (4202, u64::MAX - 20),
                    (SwapBridgeLeftPot::account_id(), 10),
                ],
            },
            balances_right: pallet_balances::GenesisConfig {
                balances: vec![(4211, 20), (SwapBridgeRightPot::account_id(), 100)],
            },
            swap_bridge_left_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Initialized,
            },
            swap_bridge_right_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Initialized,
            },
            ..Default::default()
        };

        new_test_ext_with(config).execute_with(move || {});
    })
}

/// This test verifies that `balanced_value` function works in the happy path.
#[test]
fn balanced_value_works() {
    with_runtime_lock(|| {
        let left_balances = vec![10, 20, 30, 40];
        let right_balances = vec![20, 40, 60, 80, 100];

        let expected_left_bridge_balance =
            right_balances.iter().sum::<u64>() + EXISTENTIAL_DEPOSIT_LEFT;
        let expected_right_bridge_balance =
            left_balances.iter().sum::<u64>() + EXISTENTIAL_DEPOSIT_RIGHT;

        assert_eq!(
            expected_left_bridge_balance,
            Balanced::<Test, BridgeInstanceRightToLeftSwap>::genesis_swappable_balance_at_from(
                right_balances
            )
            .unwrap()
        );
        assert_eq!(
            expected_right_bridge_balance,
            Balanced::<Test, BridgeInstanceLeftToRightSwap>::genesis_swappable_balance_at_from(
                left_balances
            )
            .unwrap()
        );
    })
}

/// This test verifies that `balanced_value` function fails in case overflow error happens.
#[test]
fn balanced_value_fails_overflow() {
    with_runtime_lock(|| {
        let left_balances = vec![10, 20, 30, u64::MAX];
        assert_eq!(
            BalancedError::Arithmetic(ArithmeticError::Overflow),
            Balanced::<Test, BridgeInstanceLeftToRightSwap>::genesis_swappable_balance_at_from(
                left_balances
            )
            .unwrap_err()
        );
    })
}
