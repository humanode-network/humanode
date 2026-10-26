//! The tests for the pallet.

use fp_evm::FeeCalculator;
use frame_support::{
    assert_noop, assert_ok, parameter_types,
    sp_runtime::{FixedPointNumber, FixedU128},
    traits::{ConstU128, Get, GetStorageVersion, OnRuntimeUpgrade},
    weights::{ConstantMultiplier, RuntimeDbWeight, Weight, WeightToFee},
};
use sp_core::U256;

use crate::{
    mock::{
        new_test_ext, new_test_ext_with, new_test_ext_without_pallet_genesis, HumanodeFeeParams,
        RuntimeOrigin, Test,
    },
    *,
};

parameter_types! {
    pub const MigrationNativeFeeMultiplier: FixedU128 = FixedU128::from_u32(10);
    pub const MigrationEvmFeeMultiplier: FixedU128 = FixedU128::from_u32(15);
}

/// The migration under test, with the multipliers to enact.
type Migration = migrations::v1::EnactFeeMultipliers<
    Test,
    MigrationNativeFeeMultiplier,
    MigrationEvmFeeMultiplier,
>;

/// This test verifies that the test setup for the genesis is adequate.
#[test]
fn basic_setup_works() {
    new_test_ext().execute_with(|| {
        // Check the initial state.
        assert_eq!(<NativeFeeMultiplier<Test>>::get(), FixedU128::from_u32(1));
        assert_eq!(<EvmFeeMultiplier<Test>>::get(), FixedU128::from_u32(2));
        assert_eq!(
            HumanodeFeeParams::on_chain_storage_version(),
            STORAGE_VERSION
        );
    });
}

/// This test verifies that the genesis config sets the multipliers.
#[test]
fn genesis_config_works() {
    let genesis_config = mock::GenesisConfig {
        system: Default::default(),
        humanode_fee_params: GenesisConfig {
            native_fee_multiplier: FixedU128::from_u32(3),
            evm_fee_multiplier: FixedU128::from_u32(4),
        },
    };
    new_test_ext_with(genesis_config).execute_with(|| {
        assert_eq!(<NativeFeeMultiplier<Test>>::get(), FixedU128::from_u32(3));
        assert_eq!(<EvmFeeMultiplier<Test>>::get(), FixedU128::from_u32(4));
    });
}

/// This test verifies that setting the native fee multiplier works in the happy path.
#[test]
fn setting_native_fee_multiplier_works() {
    new_test_ext().execute_with(|| {
        // Check test preconditions.
        assert_eq!(<NativeFeeMultiplier<Test>>::get(), FixedU128::from_u32(1));

        // Set block number to enable events.
        mock::System::set_block_number(1);

        // Invoke the function under test.
        let multiplier = FixedU128::from_rational(5, 2);
        assert_ok!(HumanodeFeeParams::set_native_fee_multiplier(
            RuntimeOrigin::root(),
            multiplier
        ));

        // Assert state changes.
        assert_eq!(<NativeFeeMultiplier<Test>>::get(), multiplier);
        assert_eq!(<EvmFeeMultiplier<Test>>::get(), FixedU128::from_u32(2));
        mock::System::assert_has_event(mock::RuntimeEvent::HumanodeFeeParams(
            Event::NativeFeeMultiplierUpdated { multiplier },
        ));
    });
}

/// This test verifies that setting the EVM fee multiplier works in the happy path.
#[test]
fn setting_evm_fee_multiplier_works() {
    new_test_ext().execute_with(|| {
        // Check test preconditions.
        assert_eq!(<EvmFeeMultiplier<Test>>::get(), FixedU128::from_u32(2));

        // Set block number to enable events.
        mock::System::set_block_number(1);

        // Invoke the function under test.
        let multiplier = FixedU128::from_rational(5, 2);
        assert_ok!(HumanodeFeeParams::set_evm_fee_multiplier(
            RuntimeOrigin::root(),
            multiplier
        ));

        // Assert state changes.
        assert_eq!(<EvmFeeMultiplier<Test>>::get(), multiplier);
        assert_eq!(<NativeFeeMultiplier<Test>>::get(), FixedU128::from_u32(1));
        mock::System::assert_has_event(mock::RuntimeEvent::HumanodeFeeParams(
            Event::EvmFeeMultiplierUpdated { multiplier },
        ));
    });
}

/// This test verifies that the multipliers can be set to fractional values, below one included.
#[test]
fn fractional_multipliers_work() {
    new_test_ext().execute_with(|| {
        // Half of the base fee.
        let native_multiplier = FixedU128::from_rational(1, 2);
        // One permill of the base fee.
        let evm_multiplier = FixedU128::from_rational(1, 1000);

        assert_ok!(HumanodeFeeParams::set_native_fee_multiplier(
            RuntimeOrigin::root(),
            native_multiplier
        ));
        assert_ok!(HumanodeFeeParams::set_evm_fee_multiplier(
            RuntimeOrigin::root(),
            evm_multiplier
        ));

        assert_eq!(<NativeFeeMultiplier<Test>>::get(), native_multiplier);
        assert_eq!(<EvmFeeMultiplier<Test>>::get(), evm_multiplier);

        // The multipliers scale the integer amounts fractionally.
        assert_eq!(native_multiplier.saturating_mul_int(1_000u128), 500);
        assert_eq!(evm_multiplier.saturating_mul_int(1_000_000u128), 1_000);
    });
}

/// This test verifies that setting the native fee multiplier fails with a non-root origin.
#[test]
fn setting_native_fee_multiplier_requires_root() {
    new_test_ext().execute_with(|| {
        let multiplier = FixedU128::from_u32(5);

        // Invoke the function under test and assert it fails.
        assert_noop!(
            HumanodeFeeParams::set_native_fee_multiplier(RuntimeOrigin::signed(1), multiplier),
            frame_support::dispatch::DispatchError::BadOrigin
        );
        assert_noop!(
            HumanodeFeeParams::set_native_fee_multiplier(RuntimeOrigin::none(), multiplier),
            frame_support::dispatch::DispatchError::BadOrigin
        );
    });
}

/// This test verifies that setting the EVM fee multiplier fails with a non-root origin.
#[test]
fn setting_evm_fee_multiplier_requires_root() {
    new_test_ext().execute_with(|| {
        let multiplier = FixedU128::from_u32(5);

        // Invoke the function under test and assert it fails.
        assert_noop!(
            HumanodeFeeParams::set_evm_fee_multiplier(RuntimeOrigin::signed(1), multiplier),
            frame_support::dispatch::DispatchError::BadOrigin
        );
        assert_noop!(
            HumanodeFeeParams::set_evm_fee_multiplier(RuntimeOrigin::none(), multiplier),
            frame_support::dispatch::DispatchError::BadOrigin
        );
    });
}

/// This test verifies that the native weight to fee conversion scales the baseline by the native
/// fee multiplier.
#[test]
fn native_weight_to_fee_works() {
    new_test_ext().execute_with(|| {
        type Baseline = ConstantMultiplier<u128, ConstU128<1000>>;
        type Native = NativeWeightToFee<Test, Baseline>;

        let weight = Weight::from_parts(7, 0);
        assert_eq!(Baseline::weight_to_fee(&weight), 7000);

        // The default multiplier keeps the baseline fee as is.
        assert_eq!(Native::weight_to_fee(&weight), 7000);

        assert_ok!(HumanodeFeeParams::set_native_fee_multiplier(
            RuntimeOrigin::root(),
            FixedU128::from_u32(3)
        ));
        assert_eq!(Native::weight_to_fee(&weight), 21_000);

        assert_ok!(HumanodeFeeParams::set_native_fee_multiplier(
            RuntimeOrigin::root(),
            FixedU128::from_rational(1, 2)
        ));
        assert_eq!(Native::weight_to_fee(&weight), 3500);
    });
}

/// This test verifies that the EVM fee calculator scales the baseline fee per gas by the EVM fee
/// multiplier.
#[test]
fn evm_fee_calculator_works() {
    new_test_ext().execute_with(|| {
        type Evm = EvmFeeCalculator<Test, ConstU128<1000>>;

        // Reading the multiplier is one DB read.
        let expected_weight =
            <<Test as frame_system::Config>::DbWeight as Get<RuntimeDbWeight>>::get().reads(1);

        // The default multiplier doubles the baseline.
        assert_eq!(Evm::min_gas_price(), (U256::from(2000), expected_weight));

        assert_ok!(HumanodeFeeParams::set_evm_fee_multiplier(
            RuntimeOrigin::root(),
            FixedU128::from_u32(3)
        ));
        assert_eq!(Evm::min_gas_price(), (U256::from(3000), expected_weight));

        assert_ok!(HumanodeFeeParams::set_evm_fee_multiplier(
            RuntimeOrigin::root(),
            FixedU128::from_rational(1, 2)
        ));
        assert_eq!(Evm::min_gas_price(), (U256::from(500), expected_weight));
    });
}

/// This test verifies that the v1 migration enacts the multipliers on a chain where the pallet
/// storage has not been populated.
#[test]
fn migration_v1_enacts_multipliers() {
    new_test_ext_without_pallet_genesis().execute_with(|| {
        // Check test preconditions.
        assert_eq!(HumanodeFeeParams::on_chain_storage_version(), 0);
        assert!(!<NativeFeeMultiplier<Test>>::exists());
        assert!(!<EvmFeeMultiplier<Test>>::exists());

        // Invoke the function under test.
        let weight = Migration::on_runtime_upgrade();

        // Assert state changes.
        assert_eq!(<NativeFeeMultiplier<Test>>::get(), FixedU128::from_u32(10));
        assert_eq!(<EvmFeeMultiplier<Test>>::get(), FixedU128::from_u32(15));
        assert_eq!(HumanodeFeeParams::on_chain_storage_version(), 1);
        assert_eq!(
            weight,
            <<Test as frame_system::Config>::DbWeight as Get<RuntimeDbWeight>>::get()
                .reads_writes(1, 3)
        );
    });
}

/// This test verifies that the v1 migration does nothing on a chain where the pallet storage
/// is already at the target version.
#[test]
fn migration_v1_is_noop_when_migrated() {
    new_test_ext().execute_with(|| {
        // Check test preconditions.
        assert_eq!(HumanodeFeeParams::on_chain_storage_version(), 1);

        // Invoke the function under test.
        let weight = Migration::on_runtime_upgrade();

        // Assert the state is untouched.
        assert_eq!(<NativeFeeMultiplier<Test>>::get(), FixedU128::from_u32(1));
        assert_eq!(<EvmFeeMultiplier<Test>>::get(), FixedU128::from_u32(2));
        assert_eq!(HumanodeFeeParams::on_chain_storage_version(), 1);
        assert_eq!(
            weight,
            <<Test as frame_system::Config>::DbWeight as Get<RuntimeDbWeight>>::get().reads(1)
        );
    });
}

/// This test verifies that the v1 migration try-runtime checks pass through the migration.
#[cfg(feature = "try-runtime")]
#[test]
fn migration_v1_try_runtime_checks_work() {
    new_test_ext_without_pallet_genesis().execute_with(|| {
        let state = Migration::pre_upgrade().unwrap();
        Migration::on_runtime_upgrade();
        assert_ok!(Migration::post_upgrade(state));
    });

    new_test_ext().execute_with(|| {
        let state = Migration::pre_upgrade().unwrap();
        Migration::on_runtime_upgrade();
        assert_ok!(Migration::post_upgrade(state));
    });
}
