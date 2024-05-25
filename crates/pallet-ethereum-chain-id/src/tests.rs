use crate::{self as pallet_ethereum_chain_id, mock::*};

/// This test verifies that genesis initialization properly assigns the state.
#[test]
fn genesis_build() {
    // Prepare some sample data and a config.
    let chain_id = 2020;
    let config = pallet_ethereum_chain_id::GenesisConfig { chain_id };

    // Build the state from the config.
    new_test_ext_with(config).execute_with(move || {
        // Assert the state.
        assert_eq!(EthereumChainId::chain_id(), chain_id);
    })
}
