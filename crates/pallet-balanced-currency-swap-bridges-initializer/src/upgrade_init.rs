//! Initialization of the bridge pot accounts on runtime upgrade.

use frame_support::{
    log::{error, info},
    pallet_prelude::*,
    traits::OnRuntimeUpgrade,
};
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

use crate::{
    Config, LastForceRebalanceAskCounter, LastInitializerVersion, Pallet,
    CURRENT_BRIDGES_INITIALIZER_VERSION,
};

/// Execute upgrade init.
pub struct UpgradeInit<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for UpgradeInit<T> {
    fn on_runtime_upgrade() -> Weight {
        let last_initializer_version = <LastInitializerVersion<T>>::get();
        let last_force_rebalance_ask_counter = <LastForceRebalanceAskCounter<T>>::get();
        let current_force_rebalance_ask_counter = T::ForceRebalanceAskCounter::get();

        let mut weight = T::DbWeight::get().reads(2);

        let is_version_mismatch = last_initializer_version != CURRENT_BRIDGES_INITIALIZER_VERSION;
        let is_forced = last_force_rebalance_ask_counter < current_force_rebalance_ask_counter;

        if is_version_mismatch || is_forced {
            match Pallet::<T>::initialize() {
                Ok(w) => weight.saturating_accrue(w),
                Err(err) => error!("error during bridges initialization: {err:?}"),
            }

            <LastInitializerVersion<T>>::put(CURRENT_BRIDGES_INITIALIZER_VERSION);
            <LastForceRebalanceAskCounter<T>>::put(current_force_rebalance_ask_counter);
            weight.saturating_accrue(T::DbWeight::get().writes(2));
        } else {
            info!("Nothing to do. This runtime upgrade probably should be removed");
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
        use frame_support::{storage_root, StateVersion};

        let storage_root_before = storage_root(StateVersion::V1);

        if !Pallet::<T>::is_balanced()? {
            return Err("currencies are not balanced");
        }

        assert_eq!(storage_root_before, storage_root(StateVersion::V1));

        Ok(())
    }
}
