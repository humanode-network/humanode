use crate::{self as pallet_chain_properties, mock::*};

/// This test verifies that genesis initialization properly assignes the state.
#[test]
fn genesis_build() {
    // Prepare some sample data and a config.
    let ss58_prefix = 2020;
    let config = pallet_chain_properties::GenesisConfig { ss58_prefix };

    // Build the state from the config.
    new_test_ext_with(config).execute_with(move || {
        // Assert the state.
        assert_eq!(NativeChainId::ss58_prefix(), ss58_prefix);
    })
}
