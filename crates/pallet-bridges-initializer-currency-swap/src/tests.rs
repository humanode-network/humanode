use frame_support::traits::{Currency, OnRuntimeUpgrade};

use crate::mock::{new_test_ext_with, v1, v2::*, with_runtime_lock, *};

#[test]
fn initialization_bridges_ed_works() {
    with_runtime_lock(|| {
        let treasury = (NativeTreasury::get(), 1450);
        let alice = (4201, 20);
        let bob = (4203, 30);
        let swap_bridge_native_evm = (
            SwapBridgeNativeToEvmPot::account_id(),
            EXISTENTIAL_DEPOSIT_NATIVE,
        );

        let lion = (4211, 200);
        let dog = (4212, 300);
        let cat = (4213, 400);
        let fish = (4214, 500);
        let swap_bridge_evm_native = (
            SwapBridgeEvmToNativePot::account_id(),
            EXISTENTIAL_DEPOSIT_EVM,
        );

        let config = GenesisConfig {
            balances: pallet_balances::GenesisConfig {
                balances: vec![treasury, alice, bob, swap_bridge_native_evm],
            },
            evm_balances: pallet_balances::GenesisConfig {
                balances: vec![lion, dog, cat, fish, swap_bridge_evm_native],
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
                Balances::total_balance(&SwapBridgeNativeToEvmPot::account_id()),
                lion.1 + dog.1 + cat.1 + fish.1 + EXISTENTIAL_DEPOSIT_NATIVE
            );
            assert_eq!(
                Balances::total_balance(&NativeTreasury::get()),
                treasury.1 - (lion.1 + dog.1 + cat.1 + fish.1)
            );
            assert_eq!(
                EvmBalances::total_balance(&SwapBridgeEvmToNativePot::account_id(),),
                Balances::total_balance(&NativeTreasury::get())
                    + alice.1
                    + bob.1
                    + EXISTENTIAL_DEPOSIT_EVM
            );
        });
    })
}

#[test]
fn initialization_bridges_ed_delta_works() {
    with_runtime_lock(|| {
        let treasury = (NativeTreasury::get(), 1450);
        let alice = (4201, 20);
        let bob = (4203, 30);
        let native_bridge_delta = 100;
        let swap_bridge_native_evm = (
            SwapBridgeNativeToEvmPot::account_id(),
            EXISTENTIAL_DEPOSIT_NATIVE + native_bridge_delta,
        );

        let lion = (4211, 200);
        let dog = (4212, 300);
        let cat = (4213, 400);
        let fish = (4214, 500);
        let evm_bridge_delta = 50;
        let swap_bridge_evm_native = (
            SwapBridgeEvmToNativePot::account_id(),
            EXISTENTIAL_DEPOSIT_EVM + evm_bridge_delta,
        );

        let config = GenesisConfig {
            balances: pallet_balances::GenesisConfig {
                balances: vec![treasury, alice, bob, swap_bridge_native_evm],
            },
            evm_balances: pallet_balances::GenesisConfig {
                balances: vec![lion, dog, cat, fish, swap_bridge_evm_native],
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
                Balances::total_balance(&SwapBridgeNativeToEvmPot::account_id()),
                lion.1 + dog.1 + cat.1 + fish.1 + EXISTENTIAL_DEPOSIT_NATIVE
            );
            assert_eq!(
                Balances::total_balance(&NativeTreasury::get()),
                treasury.1 - (lion.1 + dog.1 + cat.1 + fish.1) + native_bridge_delta
            );
            assert_eq!(
                EvmBalances::total_balance(&SwapBridgeEvmToNativePot::account_id(),),
                Balances::total_balance(&NativeTreasury::get())
                    + alice.1
                    + bob.1
                    + EXISTENTIAL_DEPOSIT_EVM
            );
        });
    })
}

#[test]
#[should_panic = "error during bridges initialization: Arithmetic(Overflow)"]
fn initialization_fails_overflow() {
    with_runtime_lock(|| {
        let treasury = (NativeTreasury::get(), EXISTENTIAL_DEPOSIT_NATIVE);
        let swap_bridge_native_evm = (
            SwapBridgeNativeToEvmPot::account_id(),
            u64::MAX - EXISTENTIAL_DEPOSIT_NATIVE,
        );

        let swap_bridge_evm_native = (
            SwapBridgeEvmToNativePot::account_id(),
            EXISTENTIAL_DEPOSIT_EVM,
        );

        let config = GenesisConfig {
            balances: pallet_balances::GenesisConfig {
                balances: vec![treasury, swap_bridge_native_evm],
            },
            evm_balances: pallet_balances::GenesisConfig {
                balances: vec![swap_bridge_evm_native],
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
        let treasury = (NativeTreasury::get(), EXISTENTIAL_DEPOSIT_NATIVE + 10);
        let swap_bridge_native_evm = (
            SwapBridgeNativeToEvmPot::account_id(),
            EXISTENTIAL_DEPOSIT_NATIVE,
        );

        let lion = (4211, 200);
        let dog = (4212, 300);
        let cat = (4213, 400);
        let fish = (4214, 500);
        let evm_bridge_delta = 50;
        let swap_bridge_evm_native = (
            SwapBridgeEvmToNativePot::account_id(),
            EXISTENTIAL_DEPOSIT_EVM + evm_bridge_delta,
        );

        let config = GenesisConfig {
            balances: pallet_balances::GenesisConfig {
                balances: vec![treasury, swap_bridge_native_evm],
            },
            evm_balances: pallet_balances::GenesisConfig {
                balances: vec![lion, dog, cat, fish, swap_bridge_evm_native],
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
        let treasury = (NativeTreasury::get(), 1450);
        let alice = (4201, 20);
        let bob = (4203, 30);

        let lion = (4211, 200);
        let dog = (4212, 300);
        let cat = (4213, 400);
        let fish = (4214, 500);

        let config = v1::GenesisConfig {
            balances: pallet_balances::GenesisConfig {
                balances: vec![treasury, alice, bob],
            },
            evm_balances: pallet_balances::GenesisConfig {
                balances: vec![lion, dog, cat, fish],
            },
            ..Default::default()
        };

        let mut v1_ext = new_test_ext_with(config);
        v1_ext.execute_with(move || {
            // Do runtime upgrade hook.
            AllPalletsWithoutSystem::on_runtime_upgrade();

            // Verify bridges initialization result.
            assert!(EvmNativeBridgesInitializer::is_balanced().unwrap());
            assert_eq!(
                Balances::total_balance(&SwapBridgeNativeToEvmPot::account_id()),
                lion.1 + dog.1 + cat.1 + fish.1 + EXISTENTIAL_DEPOSIT_NATIVE
            );
            assert_eq!(
                Balances::total_balance(&NativeTreasury::get()),
                treasury.1 - (lion.1 + dog.1 + cat.1 + fish.1 + EXISTENTIAL_DEPOSIT_NATIVE)
            );
            assert_eq!(
                EvmBalances::total_balance(&SwapBridgeEvmToNativePot::account_id(),),
                Balances::total_balance(&NativeTreasury::get())
                    + alice.1
                    + bob.1
                    + EXISTENTIAL_DEPOSIT_EVM
            );
        });
    })
}
