use frame_support::traits::OnRuntimeUpgrade;

use crate::{mock::*, DUMMY_CODE};

/// This test verifies that genesis initialization properly assignes the state.
#[test]
fn genesis_build() {
    // Build the state from the config.
    new_test_ext_with(v1::GenesisConfig::default()).execute_with(move || {
        // Assert the state.
        for precompile_address in &v1::PrecompilesAddresses::get() {
            assert_eq!(
                v1::Evm::account_codes(precompile_address),
                DUMMY_CODE.to_vec()
            );
            assert!(v1::EvmSystem::account_exists(precompile_address));
        }
        assert_eq!(
            v1::DummyPrecompilesCode::creation_version(),
            crate::CURRENT_CREATION_VERSION
        );
        assert_eq!(v1::DummyPrecompilesCode::force_update_ask_counter(), 0);
    })
}

/// This test simulates runtime upgrade operation by using different mocked runtime versions and
/// verifies that precompiles addresses have been created as expected for `on_runtime_upgrade` call.
///
/// - v0: simple evm based version.
/// - v1: v0 with dummy precompiles code pallet.
#[test]
fn runtime_upgrade() {
    with_runtime_lock(|| {
        new_test_ext_with(v0::GenesisConfig::default()).execute_with(move || {
            // Check test preconditions.
            for precompile_address in &v1::PrecompilesAddresses::get() {
                assert!(v0::Evm::account_codes(precompile_address).is_empty());
                assert!(!v0::EvmSystem::account_exists(precompile_address));
            }
            assert_eq!(v1::DummyPrecompilesCode::creation_version(), 0);
            assert_eq!(v1::DummyPrecompilesCode::force_update_ask_counter(), 0);

            // Do runtime upgrade hook.
            v1::AllPalletsWithoutSystem::on_runtime_upgrade();

            // Verify precompiles addresses creation.
            for precompile_address in &v1::PrecompilesAddresses::get() {
                assert_eq!(
                    v1::Evm::account_codes(precompile_address),
                    DUMMY_CODE.to_vec()
                );
                assert!(v1::EvmSystem::account_exists(precompile_address));
            }
            assert_eq!(
                v1::DummyPrecompilesCode::creation_version(),
                crate::CURRENT_CREATION_VERSION
            );
            assert_eq!(v1::DummyPrecompilesCode::force_update_ask_counter(), 0);
        });
    })
}
