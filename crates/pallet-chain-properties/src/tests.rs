// Allow simple integer arithmetic in tests.
#![allow(clippy::integer_arithmetic)]

use frame_support::traits::Get;

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
        assert_eq!(ChainProperties::ss58_prefix(), ss58_prefix);
        assert_eq!(
            <<Test as frame_system::Config>::SS58Prefix as Get<u16>>::get(),
            ss58_prefix
        );
    })
}
