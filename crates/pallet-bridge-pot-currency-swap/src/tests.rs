use frame_support::sp_runtime::{ArithmeticError, DispatchError};

use crate::mock::*;

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

/// This test verifies that the genesis builder correctly ensures genesis bridge pot balances
/// values that are balanced in case swappable balance fully belongs to one of the bridges.
#[test]
fn genesis_balanced_true_full_swappable_to_one_bridge() {
    with_runtime_lock(|| {
        let swappable_balance = 1000;

        let config = GenesisConfig {
            balances_left: pallet_balances::GenesisConfig {
                balances: vec![
                    (4201, swappable_balance),
                    (SwapBridgeLeftPot::account_id(), EXISTENTIAL_DEPOSIT_LEFT),
                ],
            },
            balances_right: pallet_balances::GenesisConfig {
                balances: vec![(
                    SwapBridgeRightPot::account_id(),
                    swappable_balance + EXISTENTIAL_DEPOSIT_RIGHT,
                )],
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

/// This test verifies that the genesis builder correctly ensures genesis bridge pot balances
/// values that are balanced in case swappable balance is zero.
#[test]
fn genesis_balanced_true_swappable_zero() {
    with_runtime_lock(|| {
        let config = GenesisConfig {
            balances_left: pallet_balances::GenesisConfig {
                balances: vec![(SwapBridgeLeftPot::account_id(), EXISTENTIAL_DEPOSIT_LEFT)],
            },
            balances_right: pallet_balances::GenesisConfig {
                balances: vec![(SwapBridgeRightPot::account_id(), EXISTENTIAL_DEPOSIT_RIGHT)],
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

/// This test verifies that the genesis builder panics in case bridge pot balances values
/// are not balanced.
#[test]
#[should_panic = "invalid bridge balance value: got 100, expected 30"]
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
#[should_panic = "error during bridge balance calculation: Arithmetic(Overflow)"]
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

/// This test verifies that `genesis_bridge_to_balance` function works in the happy path.
#[test]
fn genesis_bridge_to_balance_works() {
    with_runtime_lock(|| {
        let genesis_left_balances = vec![10, 20, 30, 40];
        let genesis_right_balances = vec![20, 40, 60, 80, 100];

        let expected_left_bridge_balance =
            genesis_right_balances.iter().sum::<u64>() + EXISTENTIAL_DEPOSIT_LEFT;
        let expected_right_bridge_balance =
            genesis_left_balances.iter().sum::<u64>() + EXISTENTIAL_DEPOSIT_RIGHT;

        assert_eq!(
            expected_right_bridge_balance,
            SwapBridgeLeft::genesis_bridge_to_balance(genesis_left_balances).unwrap()
        );
        assert_eq!(
            expected_left_bridge_balance,
            SwapBridgeRight::genesis_bridge_to_balance(genesis_right_balances).unwrap()
        );
    })
}

/// This test verifies that `genesis_bridge_to_balance` function fails in case overflow error happens.
#[test]
fn genesis_bridge_to_balance_fails_overflow() {
    with_runtime_lock(|| {
        let genesis_left_balances = vec![10, 20, 30, u64::MAX];
        assert_eq!(
            DispatchError::Arithmetic(ArithmeticError::Overflow),
            SwapBridgeRight::genesis_bridge_to_balance(genesis_left_balances).unwrap_err()
        );
    })
}

/// This test verifies that `genesis_bridge_to_balance` function calculates genesis bridge balances
/// as expected to pass genesis validation.
#[test]
fn genesis_bridge_to_balance_passes_genesis_validation() {
    with_runtime_lock(|| {
        let left_basic_accounts = vec![(4201, 10), (4202, 20), (4203, 30), (4204, 40)];
        let right_basic_accounts =
            vec![(4211, 20), (4212, 40), (4213, 60), (4214, 80), (4215, 100)];

        let config = GenesisConfig {
            balances_left: pallet_balances::GenesisConfig {
                balances: vec![
                    (4201, 10),
                    (4202, 20),
                    (4203, 30),
                    (4204, 40),
                    (
                        SwapBridgeLeftPot::account_id(),
                        SwapBridgeRight::genesis_bridge_to_balance(
                            left_basic_accounts
                                .iter()
                                .map(|(_, balance)| balance)
                                .copied(),
                        )
                        .unwrap(),
                    ),
                ],
            },
            balances_right: pallet_balances::GenesisConfig {
                balances: vec![
                    (4211, 20),
                    (4212, 40),
                    (4213, 60),
                    (4214, 80),
                    (4215, 100),
                    (
                        SwapBridgeRightPot::account_id(),
                        SwapBridgeLeft::genesis_bridge_to_balance(
                            right_basic_accounts
                                .iter()
                                .map(|(_, balance)| balance)
                                .copied(),
                        )
                        .unwrap(),
                    ),
                ],
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
