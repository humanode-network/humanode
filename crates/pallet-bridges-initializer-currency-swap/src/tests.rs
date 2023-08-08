use frame_support::traits::Currency;

use crate::mock::*;

#[test]
fn initialization_works() {
    with_runtime_lock(|| {
        let config = GenesisConfig {
            balances: pallet_balances::GenesisConfig {
                balances: vec![
                    (NativeTreasury::get(), 1450),
                    (4201, 20),
                    (4203, 30),
                    (
                        SwapBridgeNativeToEvmPot::account_id(),
                        EXISTENTIAL_DEPOSIT_NATIVE,
                    ),
                ],
            },
            evm_balances: pallet_balances::GenesisConfig {
                balances: vec![
                    (4211, 200),
                    (4212, 300),
                    (4213, 400),
                    (4214, 500),
                    (
                        SwapBridgeEvmToNativePot::account_id(),
                        EXISTENTIAL_DEPOSIT_EVM,
                    ),
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
                Balances::total_balance(&SwapBridgeNativeToEvmPot::account_id()),
                200 + 300 + 400 + 500 + EXISTENTIAL_DEPOSIT_NATIVE
            );
            assert_eq!(Balances::total_balance(&NativeTreasury::get()), 50);
            assert_eq!(
                EvmBalances::total_balance(&SwapBridgeEvmToNativePot::account_id(),),
                50 + 20 + 30 + EXISTENTIAL_DEPOSIT_EVM
            );
        });
    })
}
