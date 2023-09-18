use sp_core::H160;

use crate::{self as pallet_evm_precompiles_constant_accounts, mock::*};

#[test]
fn genesis_build() {
    // Prepare some sample data and a config.
    let precompile_a = H160::from_low_u64_be(2020);
    let precompile_b = H160::from_low_u64_be(2040);

    let config = GenesisConfig {
        evm_precompiles_constant_accounts: {
            pallet_evm_precompiles_constant_accounts::GenesisConfig {
                precompiles: vec![precompile_a, precompile_b],
            }
        },
        ..Default::default()
    };

    // Build the state from the config.
    new_test_ext_with(config).execute_with(move || {
        // Assert the state.
        assert_eq!(Evm::account_codes(precompile_a), vec![1, 2, 3]);
    })
}
