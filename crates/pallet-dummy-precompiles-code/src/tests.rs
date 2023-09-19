use crate::{mock::*, DUMMY_CODE};

/// This test verifies that genesis initialization properly assignes the state.
#[test]
fn genesis_build() {
    // Build the state from the config.
    new_test_ext().execute_with(move || {
        // Assert the state.
        for precompile_address in &PrecompilesAddresses::get() {
            assert_eq!(
                Evm::account_codes(precompile_address),
                DUMMY_CODE.as_bytes().to_vec()
            );
            assert!(EvmSystem::account_exists(precompile_address));
        }
    })
}
