use frame_support::traits::Currency;

use crate::mock::*;

#[test]
fn works() {
    with_runtime_lock(|| {
        let config = GenesisConfig {
            balances: pallet_balances::GenesisConfig {
                balances: vec![(4200, 1450), (4201, 20), (4203, 30)],
            },
            evm_balances: pallet_balances::GenesisConfig {
                balances: vec![(4211, 200), (4212, 300), (4213, 400), (4214, 500)],
            },
            swap_bridge_native_to_evm_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Unchecked,
            },
            swap_bridge_evm_to_native_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Unchecked,
            },
            ..Default::default()
        };
        new_test_ext_with(config).execute_with(move || {
            assert_eq!(
                Balances::total_balance(&SwapBridgeNativeToEvmPot::account_id()),
                1410
            );
            assert_eq!(Balances::total_balance(&4200), 40);
            assert_eq!(
                EvmBalances::total_balance(&SwapBridgeNativeToEvmPot::account_id(),),
                40 + 20 + 30 + EXISTENTIAL_DEPOSIT_EVM
            );
        });
    })
}
