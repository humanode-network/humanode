//! Upgrade init implementation.

#[cfg(feature = "try-runtime")]
use frame_support::sp_std::vec::Vec;
use frame_support::{
    sp_std,
    sp_tracing::info,
    traits::{Get, OnRuntimeUpgrade},
    weights::Weight,
};

use crate::{
    Config, LastExecutionVersion, LastForceExecuteAskCounter, Pallet, CURRENT_EXECUTION_VERSION,
};

/// Execute upgrade init.
pub struct UpgradeInit<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for UpgradeInit<T> {
    fn on_runtime_upgrade() -> Weight {
        let last_execution_version = Pallet::<T>::last_execution_version();
        let last_force_execute_ask_counter = Pallet::<T>::last_force_execute_ask_counter();

        let current_force_execute_ask_counter = T::ForceExecuteAskCounter::get();
        let mut weight = T::DbWeight::get().reads(2);

        let is_version_mismatch = last_execution_version != CURRENT_EXECUTION_VERSION;
        let is_forced = last_force_execute_ask_counter < current_force_execute_ask_counter;

        if is_version_mismatch || is_forced {
            weight.saturating_accrue(Pallet::<T>::precompiles_addresses_add_dummy_code());

            <LastExecutionVersion<T>>::put(CURRENT_EXECUTION_VERSION);
            <LastForceExecuteAskCounter<T>>::put(current_force_execute_ask_counter);
            weight.saturating_accrue(T::DbWeight::get().writes(2));
        } else {
            info!(message = "Nothing to do. This runtime upgrade probably should be removed");
        }

        weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        // Do nothing.
        Ok(Vec::new())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
        use crate::DUMMY_CODE;

        let mut not_created_precompiles = Vec::new();

        for precompile_address in &T::PrecompilesAddresses::get() {
            let code = pallet_evm::AccountCodes::<T>::get(*precompile_address);
            if code != DUMMY_CODE {
                not_created_precompiles.push(*precompile_address);
            }
        }

        if !not_created_precompiles.is_empty() {
            return Err("precompiles not created properly: {:not_created_precompiles}");
        }

        Ok(())
    }
}
