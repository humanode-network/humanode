use sp_core::H160;

use crate::{self as pallet_evm_precompiles_constant_accounts, mock::*};

#[test]
fn genesis_build() {
    // Prepare some sample data and a config.
    let precompile_a = (H160::from_low_u64_be(2020), vec![1, 2, 3]);
    let precompile_b = (H160::from_low_u64_be(2040), vec![1, 2, 3]);

    let config = GenesisConfig {
        evm_precompiles_constant_accounts: {
            pallet_evm_precompiles_constant_accounts::GenesisConfig {
                precompiles: vec![precompile_a.clone(), precompile_b.clone()],
            }
        },
        ..Default::default()
    };

    // Build the state from the config.
    new_test_ext_with(config).execute_with(move || {
        // Assert the state.
        assert_eq!(Evm::account_codes(precompile_a.0), precompile_a.1);
        assert_eq!(Evm::account_codes(precompile_b.0), precompile_b.1);
        assert!(EvmSystem::account_exists(&precompile_a.0));
        assert!(EvmSystem::account_exists(&precompile_b.0));
    })
}
