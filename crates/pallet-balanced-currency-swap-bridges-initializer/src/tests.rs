use frame_support::{
    assert_storage_noop,
    traits::{Currency, OnRuntimeUpgrade},
};

use crate::{
    mock::{new_test_ext_with, v0, v1, v2, with_runtime_lock, *},
    swappable_balance, LastForceRebalanceAskCounter, LastInitializerVersion, UpgradeInit,
};

/// This test verifies that balanced bridges initialization works in case bridge pot accounts
/// have been created with existential deposit balance values at genesis.
#[test]
fn initialization_bridges_ed_works() {
    with_runtime_lock(|| {
        let treasury = AccountInfo {
            account: NativeTreasury::get(),
            balance: 1450,
        };
        let swap_bridge_native_evm = AccountInfo {
            account: v1::SwapBridgeNativeToEvmPot::account_id(),
            balance: EXISTENTIAL_DEPOSIT_NATIVE,
        };

        let swap_bridge_evm_native = AccountInfo {
            account: v1::SwapBridgeEvmToNativePot::account_id(),
            balance: EXISTENTIAL_DEPOSIT_EVM,
        };

        let config = v1::GenesisConfig {
            balances: pallet_balances::GenesisConfig {
                balances: vec![
                    treasury.into(),
                    ALICE.into(),
                    BOB.into(),
                    swap_bridge_native_evm.into(),
                ],
            },
            evm_balances: pallet_balances::GenesisConfig {
                balances: vec![
                    LION.into(),
                    DOG.into(),
                    CAT.into(),
                    FISH.into(),
                    swap_bridge_evm_native.into(),
                ],
            },
            swap_bridge_native_to_evm_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Initialized,
            },
            swap_bridge_evm_to_native_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Initialized,
            },
            ..Default::default()
        };
        new_test_ext_with(config).execute_with(move || {
            assert_eq!(
                <LastInitializerVersion<v1::Test>>::get(),
                CURRENT_BRIDGES_INITIALIZER_VERSION
            );
            assert_eq!(
                <LastForceRebalanceAskCounter<v1::Test>>::get(),
                v1::FORCE_REBALANCE_ASK_COUNTER
            );
            assert_eq!(
                v1::Balances::total_balance(&v1::SwapBridgeNativeToEvmPot::account_id()),
                LION.balance
                    + DOG.balance
                    + CAT.balance
                    + FISH.balance
                    + EXISTENTIAL_DEPOSIT_NATIVE
                    + (EXISTENTIAL_DEPOSIT_EVM - EXISTENTIAL_DEPOSIT_NATIVE)
            );
            assert_eq!(
                v1::Balances::total_balance(&NativeTreasury::get()),
                treasury.balance
                    - (LION.balance + DOG.balance + CAT.balance + FISH.balance)
                    - (EXISTENTIAL_DEPOSIT_EVM - EXISTENTIAL_DEPOSIT_NATIVE)
            );
            assert_eq!(
                v1::EvmBalances::total_balance(&v1::SwapBridgeEvmToNativePot::account_id(),),
                v1::Balances::total_balance(&NativeTreasury::get())
                    + ALICE.balance
                    + BOB.balance
                    + EXISTENTIAL_DEPOSIT_EVM
            );
        });
    })
}

/// This test verifies that balanced bridges initialization works in case bridge pot accounts
/// have been created with existential deposit balance values plus some deltas at genesis.
#[test]
fn initialization_bridges_ed_delta_works() {
    with_runtime_lock(|| {
        let treasury = AccountInfo {
            account: NativeTreasury::get(),
            balance: 1450,
        };
        let native_bridge_delta = 100;
        let swap_bridge_native_evm = AccountInfo {
            account: v1::SwapBridgeNativeToEvmPot::account_id(),
            balance: EXISTENTIAL_DEPOSIT_NATIVE + native_bridge_delta,
        };

        let evm_bridge_delta = 50;
        let swap_bridge_evm_native = AccountInfo {
            account: v1::SwapBridgeEvmToNativePot::account_id(),
            balance: EXISTENTIAL_DEPOSIT_EVM + evm_bridge_delta,
        };

        let config = v1::GenesisConfig {
            balances: pallet_balances::GenesisConfig {
                balances: vec![
                    treasury.into(),
                    ALICE.into(),
                    BOB.into(),
                    swap_bridge_native_evm.into(),
                ],
            },
            evm_balances: pallet_balances::GenesisConfig {
                balances: vec![
                    LION.into(),
                    DOG.into(),
                    CAT.into(),
                    FISH.into(),
                    swap_bridge_evm_native.into(),
                ],
            },
            swap_bridge_native_to_evm_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Initialized,
            },
            swap_bridge_evm_to_native_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Initialized,
            },
            ..Default::default()
        };
        new_test_ext_with(config).execute_with(move || {
            assert_eq!(
                <LastInitializerVersion<v1::Test>>::get(),
                CURRENT_BRIDGES_INITIALIZER_VERSION
            );
            assert_eq!(
                <LastForceRebalanceAskCounter<v1::Test>>::get(),
                v1::FORCE_REBALANCE_ASK_COUNTER
            );
            assert_eq!(
                v1::Balances::total_balance(&v1::SwapBridgeNativeToEvmPot::account_id()),
                LION.balance
                    + DOG.balance
                    + CAT.balance
                    + FISH.balance
                    + EXISTENTIAL_DEPOSIT_NATIVE
                    + (EXISTENTIAL_DEPOSIT_EVM - EXISTENTIAL_DEPOSIT_NATIVE)
            );
            assert_eq!(
                v1::Balances::total_balance(&NativeTreasury::get()),
                treasury.balance - (LION.balance + DOG.balance + CAT.balance + FISH.balance)
                    + native_bridge_delta
                    - (EXISTENTIAL_DEPOSIT_EVM - EXISTENTIAL_DEPOSIT_NATIVE)
            );
            assert_eq!(
                v1::EvmBalances::total_balance(&v1::SwapBridgeEvmToNativePot::account_id(),),
                v1::Balances::total_balance(&NativeTreasury::get())
                    + ALICE.balance
                    + BOB.balance
                    + EXISTENTIAL_DEPOSIT_EVM
            );
        });
    })
}

/// This test verifies idempotency of balanced bridges initialization algorithm by changing
/// balances state and applying initialization operation several times.
#[test]
fn initialization_idempotency() {
    with_runtime_lock(|| {
        let treasury = AccountInfo {
            account: NativeTreasury::get(),
            balance: 1450,
        };
        let swap_bridge_native_evm = AccountInfo {
            account: v1::SwapBridgeNativeToEvmPot::account_id(),
            balance: EXISTENTIAL_DEPOSIT_NATIVE,
        };

        let swap_bridge_evm_native = AccountInfo {
            account: v1::SwapBridgeEvmToNativePot::account_id(),
            balance: EXISTENTIAL_DEPOSIT_EVM,
        };

        let config = v1::GenesisConfig {
            balances: pallet_balances::GenesisConfig {
                balances: vec![
                    treasury.into(),
                    ALICE.into(),
                    BOB.into(),
                    swap_bridge_native_evm.into(),
                ],
            },
            evm_balances: pallet_balances::GenesisConfig {
                balances: vec![
                    LION.into(),
                    DOG.into(),
                    CAT.into(),
                    FISH.into(),
                    swap_bridge_evm_native.into(),
                ],
            },
            swap_bridge_native_to_evm_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Initialized,
            },
            swap_bridge_evm_to_native_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Initialized,
            },
            ..Default::default()
        };
        new_test_ext_with(config).execute_with(move || {
            // Verify that bridges initialization has been applied at genesis.
            assert_eq!(
                <LastInitializerVersion<v1::Test>>::get(),
                CURRENT_BRIDGES_INITIALIZER_VERSION
            );
            assert_eq!(
                <LastForceRebalanceAskCounter<v1::Test>>::get(),
                v1::FORCE_REBALANCE_ASK_COUNTER
            );
            assert!(v1::EvmNativeBridgesInitializer::is_balanced().unwrap());
            assert_eq!(
                v1::Balances::total_balance(&v1::SwapBridgeNativeToEvmPot::account_id()),
                LION.balance
                    + DOG.balance
                    + CAT.balance
                    + FISH.balance
                    + EXISTENTIAL_DEPOSIT_NATIVE
                    + (EXISTENTIAL_DEPOSIT_EVM - EXISTENTIAL_DEPOSIT_NATIVE)
            );
            assert_eq!(
                v1::Balances::total_balance(&NativeTreasury::get()),
                treasury.balance
                    - (LION.balance + DOG.balance + CAT.balance + FISH.balance)
                    - (EXISTENTIAL_DEPOSIT_EVM - EXISTENTIAL_DEPOSIT_NATIVE)
            );
            assert_eq!(
                v1::EvmBalances::total_balance(&v1::SwapBridgeEvmToNativePot::account_id(),),
                v1::Balances::total_balance(&NativeTreasury::get())
                    + ALICE.balance
                    + BOB.balance
                    + EXISTENTIAL_DEPOSIT_EVM
            );

            // Do it twice to ensure immediate reinvocation idempotency.
            assert_storage_noop!(v1::EvmNativeBridgesInitializer::initialize().unwrap());
            assert_storage_noop!(v1::EvmNativeBridgesInitializer::initialize().unwrap());

            for attempt in 0..5 {
                // Send to an existing account.
                v1::Balances::transfer(Some(ALICE.account).into(), BOB.account, 1).unwrap();
                // Create a new one account.
                v1::EvmBalances::transfer(
                    Some(FISH.account).into(),
                    5234 + attempt,
                    EXISTENTIAL_DEPOSIT_EVM,
                )
                .unwrap();

                // Initialize bridges one more time after a change.
                assert_storage_noop!(v1::EvmNativeBridgesInitializer::initialize().unwrap());
            }
        });
    })
}

/// This test verifies that balanced bridges initialization works in case genesis configuration
/// leads to 0 evm swappable balance.
#[test]
fn initialization_evm_swappable_zero() {
    with_runtime_lock(|| {
        let treasury = AccountInfo {
            account: NativeTreasury::get(),
            balance: EXISTENTIAL_DEPOSIT_NATIVE,
        };
        let swap_bridge_native_evm = AccountInfo {
            account: v1::SwapBridgeNativeToEvmPot::account_id(),
            balance: u64::MAX - EXISTENTIAL_DEPOSIT_NATIVE,
        };

        let swap_bridge_evm_native = AccountInfo {
            account: v1::SwapBridgeEvmToNativePot::account_id(),
            balance: EXISTENTIAL_DEPOSIT_EVM,
        };

        let config = v1::GenesisConfig {
            balances: pallet_balances::GenesisConfig {
                balances: vec![treasury.into(), swap_bridge_native_evm.into()],
            },
            evm_balances: pallet_balances::GenesisConfig {
                balances: vec![swap_bridge_evm_native.into()],
            },
            swap_bridge_native_to_evm_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Initialized,
            },
            swap_bridge_evm_to_native_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Initialized,
            },
            ..Default::default()
        };
        new_test_ext_with(config).execute_with(move || {
            assert_eq!(
                swappable_balance::<
                    EvmAccountId,
                    v1::EvmBalances,
                    v1::SwapBridgeEvmToNativePotAccountId,
                >()
                .unwrap(),
                0
            );
            assert_eq!(
                swappable_balance::<
                    AccountId,
                    v1::Balances,
                    v1::SwapBridgeNativeToEvmPotAccountId,
                >()
                .unwrap(),
                EXISTENTIAL_DEPOSIT_NATIVE + ((u64::MAX - EXISTENTIAL_DEPOSIT_NATIVE) - EXISTENTIAL_DEPOSIT_EVM)
            );
        });
    })
}

/// This test verifies that balanced bridges initialization fails in case genesis configuration
/// contains native treasury account with insufficient balance to properly perform initialization.
#[test]
#[should_panic = "error during bridges initialization: Module(ModuleError { index: 1, error: [2, 0, 0, 0], message: Some(\"InsufficientBalance\") })"]
fn initialization_fails_treasury_insufficient_balance() {
    with_runtime_lock(|| {
        let treasury = AccountInfo {
            account: NativeTreasury::get(),
            balance: EXISTENTIAL_DEPOSIT_NATIVE + 10,
        };
        let swap_bridge_native_evm = AccountInfo {
            account: v1::SwapBridgeNativeToEvmPot::account_id(),
            balance: EXISTENTIAL_DEPOSIT_NATIVE,
        };

        let evm_bridge_delta = 50;
        let swap_bridge_evm_native = AccountInfo {
            account: v1::SwapBridgeEvmToNativePot::account_id(),
            balance: EXISTENTIAL_DEPOSIT_EVM + evm_bridge_delta,
        };

        let config = v1::GenesisConfig {
            balances: pallet_balances::GenesisConfig {
                balances: vec![treasury.into(), swap_bridge_native_evm.into()],
            },
            evm_balances: pallet_balances::GenesisConfig {
                balances: vec![
                    LION.into(),
                    DOG.into(),
                    CAT.into(),
                    FISH.into(),
                    swap_bridge_evm_native.into(),
                ],
            },
            swap_bridge_native_to_evm_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Initialized,
            },
            swap_bridge_evm_to_native_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Initialized,
            },
            ..Default::default()
        };
        new_test_ext_with(config).execute_with(move || {});
    })
}

/// This test simulates runtime upgrade operation by using different mocked runtime versions and
/// verifies that balanced bridges initialization works as expected for `on_runtime_upgrade` call.
///
/// - v0: just contains native and evm balances.
/// - v1: v0 with balanced bridges currency swap initializer pallet.
#[test]
fn runtime_upgrade() {
    with_runtime_lock(|| {
        let treasury = AccountInfo {
            account: NativeTreasury::get(),
            balance: 1450,
        };

        let v1_config = v0::GenesisConfig {
            balances: pallet_balances::GenesisConfig {
                balances: vec![treasury.into(), ALICE.into(), BOB.into()],
            },
            evm_balances: pallet_balances::GenesisConfig {
                balances: vec![LION.into(), DOG.into(), CAT.into(), FISH.into()],
            },
            ..Default::default()
        };

        new_test_ext_with(v1_config).execute_with(move || {
            // Check test preconditions.
            assert_eq!(
                v1::Balances::total_balance(&v1::SwapBridgeNativeToEvmPot::account_id()),
                0
            );
            assert_eq!(
                v1::Balances::total_balance(&v1::SwapBridgeEvmToNativePot::account_id()),
                0
            );
            assert_eq!(<LastInitializerVersion<v1::Test>>::get(), 0);
            assert_eq!(<LastForceRebalanceAskCounter<v1::Test>>::get(), 0);

            // Do runtime upgrade hook.
            UpgradeInit::<v1::Test>::on_runtime_upgrade();

            // Verify bridges initialization result.
            assert_eq!(
                <LastInitializerVersion<v1::Test>>::get(),
                CURRENT_BRIDGES_INITIALIZER_VERSION
            );
            assert_eq!(
                <LastForceRebalanceAskCounter<v1::Test>>::get(),
                v1::FORCE_REBALANCE_ASK_COUNTER
            );
            assert!(v1::EvmNativeBridgesInitializer::is_balanced().unwrap());
            assert_eq!(
                v1::Balances::total_balance(&v1::SwapBridgeNativeToEvmPot::account_id()),
                LION.balance
                    + DOG.balance
                    + CAT.balance
                    + FISH.balance
                    + EXISTENTIAL_DEPOSIT_NATIVE
                    + (EXISTENTIAL_DEPOSIT_EVM - EXISTENTIAL_DEPOSIT_NATIVE)
            );
            assert_eq!(
                v1::Balances::total_balance(&NativeTreasury::get()),
                treasury.balance
                    - (LION.balance
                        + DOG.balance
                        + CAT.balance
                        + FISH.balance
                        + EXISTENTIAL_DEPOSIT_NATIVE)
                    - (EXISTENTIAL_DEPOSIT_EVM - EXISTENTIAL_DEPOSIT_NATIVE)
            );
            assert_eq!(
                v1::EvmBalances::total_balance(&v1::SwapBridgeEvmToNativePot::account_id()),
                v1::Balances::total_balance(&NativeTreasury::get())
                    + ALICE.balance
                    + BOB.balance
                    + EXISTENTIAL_DEPOSIT_EVM
            );

            // Get bridges balances before runtime upgrade.
            let native_evm_bridge_balance_before =
                v2::Balances::total_balance(&v2::SwapBridgeNativeToEvmPot::account_id());
            let evm_native_bridge_balance_before =
                v2::EvmBalances::total_balance(&v2::SwapBridgeEvmToNativePot::account_id());

            // Do runtime upgrade hook.
            UpgradeInit::<v2::Test>::on_runtime_upgrade();

            // Verify result.
            assert_eq!(
                <LastInitializerVersion<v2::Test>>::get(),
                CURRENT_BRIDGES_INITIALIZER_VERSION
            );
            assert_eq!(
                <LastForceRebalanceAskCounter<v2::Test>>::get(),
                v2::FORCE_REBALANCE_ASK_COUNTER
            );
            assert!(v2::EvmNativeBridgesInitializer::is_balanced().unwrap());
            assert_eq!(
                v2::Balances::total_balance(&v2::SwapBridgeNativeToEvmPot::account_id()),
                native_evm_bridge_balance_before
            );
            assert_eq!(
                v2::EvmBalances::total_balance(&v2::SwapBridgeEvmToNativePot::account_id()),
                evm_native_bridge_balance_before
            );
        });
    })
}
