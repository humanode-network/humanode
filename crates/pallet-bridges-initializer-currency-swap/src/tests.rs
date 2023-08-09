use frame_support::{
    assert_storage_noop,
    traits::{Currency, OnRuntimeUpgrade},
};

use crate::mock::{new_test_ext_with, v0, v1, with_runtime_lock, *};

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
                v1::Balances::total_balance(&v1::SwapBridgeNativeToEvmPot::account_id()),
                LION.balance
                    + DOG.balance
                    + CAT.balance
                    + FISH.balance
                    + EXISTENTIAL_DEPOSIT_NATIVE
            );
            assert_eq!(
                v1::Balances::total_balance(&NativeTreasury::get()),
                treasury.balance - (LION.balance + DOG.balance + CAT.balance + FISH.balance)
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
                v1::Balances::total_balance(&v1::SwapBridgeNativeToEvmPot::account_id()),
                LION.balance
                    + DOG.balance
                    + CAT.balance
                    + FISH.balance
                    + EXISTENTIAL_DEPOSIT_NATIVE
            );
            assert_eq!(
                v1::Balances::total_balance(&NativeTreasury::get()),
                treasury.balance - (LION.balance + DOG.balance + CAT.balance + FISH.balance)
                    + native_bridge_delta
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

#[test]
fn initialization_idempotence() {
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
            assert!(v1::EvmNativeBridgesInitializer::is_balanced().unwrap());
            assert_eq!(
                v1::Balances::total_balance(&v1::SwapBridgeNativeToEvmPot::account_id()),
                LION.balance
                    + DOG.balance
                    + CAT.balance
                    + FISH.balance
                    + EXISTENTIAL_DEPOSIT_NATIVE
            );
            assert_eq!(
                v1::Balances::total_balance(&NativeTreasury::get()),
                treasury.balance - (LION.balance + DOG.balance + CAT.balance + FISH.balance)
            );
            assert_eq!(
                v1::EvmBalances::total_balance(&v1::SwapBridgeEvmToNativePot::account_id(),),
                v1::Balances::total_balance(&NativeTreasury::get())
                    + ALICE.balance
                    + BOB.balance
                    + EXISTENTIAL_DEPOSIT_EVM
            );

            for _ in 0..5 {
                v1::Balances::transfer(Some(ALICE.account).into(), BOB.account, 1).unwrap();

                // Initialize bridges one more time.
                assert_storage_noop!(v1::EvmNativeBridgesInitializer::initialize().unwrap());
            }
        });
    })
}

#[test]
#[should_panic = "error during bridges initialization: Arithmetic(Overflow)"]
fn initialization_fails_overflow() {
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
        new_test_ext_with(config).execute_with(move || {});
    })
}

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

            // Do runtime upgrade hook.
            v1::AllPalletsWithoutSystem::on_runtime_upgrade();

            // Verify bridges initialization result.
            assert!(v1::EvmNativeBridgesInitializer::is_balanced().unwrap());
            assert_eq!(
                v1::Balances::total_balance(&v1::SwapBridgeNativeToEvmPot::account_id()),
                LION.balance
                    + DOG.balance
                    + CAT.balance
                    + FISH.balance
                    + EXISTENTIAL_DEPOSIT_NATIVE
            );
            assert_eq!(
                v1::Balances::total_balance(&NativeTreasury::get()),
                treasury.balance
                    - (LION.balance
                        + DOG.balance
                        + CAT.balance
                        + FISH.balance
                        + EXISTENTIAL_DEPOSIT_NATIVE)
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
