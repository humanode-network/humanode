use crate::mock::*;

/// This test verifies that the genesis builder correctly ensures genesis verifier result in case
/// it returns false.
#[test]
#[should_panic = "invalid genesis bridge pot currency swap related data"]
fn genesis_verifier_false() {
    with_runtime_lock(|| {
        let verify_ctx = MockGenesisVerifier::verify_context();
        verify_ctx.expect().once().return_const(false);

        let config = GenesisConfig {
            swap_bridge_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Unchecked,
            },
            ..Default::default()
        };

        new_test_ext_with(config).execute_with(move || {});

        verify_ctx.checkpoint();
    })
}

/// This test verifies that the genesis builder correctly ensures genesis verifier result in case
/// it returns true.
#[test]
fn genesis_verifier_true() {
    with_runtime_lock(|| {
        let verify_ctx = MockGenesisVerifier::verify_context();
        verify_ctx.expect().once().return_const(true);

        let config = GenesisConfig {
            swap_bridge_pot: pallet_pot::GenesisConfig {
                initial_state: pallet_pot::InitialState::Unchecked,
            },
            ..Default::default()
        };

        new_test_ext_with(config).execute_with(move || {});

        verify_ctx.checkpoint();
    })
}
