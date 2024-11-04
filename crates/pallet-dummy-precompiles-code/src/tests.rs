use frame_support::traits::OnRuntimeUpgrade;

use crate::{mock::*, UpgradeInit, DUMMY_CODE};

/// This test verifies that genesis initialization properly assigns the state.
#[test]
fn genesis_build() {
    // Build the state from the config.
    new_test_ext_with(v1::GenesisConfig::default()).execute_with(move || {
        // Assert the state.
        for precompile_address in &v1::PrecompilesAddresses::get() {
            assert_eq!(
                pallet_evm::AccountCodes::<v1::Test>::get(precompile_address),
                DUMMY_CODE.to_vec()
            );
            assert!(v1::EvmSystem::account_exists(precompile_address));
        }
        assert_eq!(
            v1::DummyPrecompilesCode::last_execution_version(),
            crate::CURRENT_EXECUTION_VERSION
        );
        assert_eq!(
            v1::DummyPrecompilesCode::last_force_execute_ask_counter(),
            0
        );
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
                assert!(pallet_evm::AccountCodes::<v0::Test>::get(precompile_address).is_empty());
                assert!(!v0::EvmSystem::account_exists(precompile_address));
            }
            assert_eq!(v1::DummyPrecompilesCode::last_execution_version(), 0);
            assert_eq!(
                v1::DummyPrecompilesCode::last_force_execute_ask_counter(),
                0
            );

            // Do runtime upgrade hook.
            UpgradeInit::<v1::Test>::on_runtime_upgrade();

            // Verify precompiles addresses creation.
            for precompile_address in &v1::PrecompilesAddresses::get() {
                assert_eq!(
                    pallet_evm::AccountCodes::<v1::Test>::get(precompile_address),
                    DUMMY_CODE.to_vec()
                );
                assert!(v1::EvmSystem::account_exists(precompile_address));
            }
            assert_eq!(
                v1::DummyPrecompilesCode::last_execution_version(),
                crate::CURRENT_EXECUTION_VERSION
            );
            assert_eq!(
                v1::DummyPrecompilesCode::last_force_execute_ask_counter(),
                0
            );
        });
    })
}
