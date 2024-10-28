//! Initialization of the bridge pot accounts on runtime upgrade.

use frame_support::pallet_prelude::*;
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

use crate::{
    Config, LastForceRebalanceAskCounter, LastInitializerVersion, Pallet,
    CURRENT_BRIDGES_INITIALIZER_VERSION,
};

/// Initialize the bridges pot accounts if required.
pub fn on_runtime_upgrade<T: Config>() -> Weight {
    let last_initializer_version = <LastInitializerVersion<T>>::get();
    let last_force_rebalance_ask_counter = <LastForceRebalanceAskCounter<T>>::get();
    let current_force_rebalance_ask_counter = T::ForceRebalanceAskCounter::get();

    let mut weight = T::DbWeight::get().reads(2);

    let is_version_mismatch = last_initializer_version != CURRENT_BRIDGES_INITIALIZER_VERSION;
    let is_forced = last_force_rebalance_ask_counter < current_force_rebalance_ask_counter;

    if is_version_mismatch || is_forced {
        match Pallet::<T>::initialize() {
            Ok(w) => weight.saturating_accrue(w),
            Err(err) => sp_tracing::error!("error during bridges initialization: {err:?}"),
        }

        <LastInitializerVersion<T>>::put(CURRENT_BRIDGES_INITIALIZER_VERSION);
        <LastForceRebalanceAskCounter<T>>::put(current_force_rebalance_ask_counter);
        weight.saturating_accrue(T::DbWeight::get().writes(2));
    }

    // Properly manage default on chain storage version as the pallet was added after genesis
    // with initial storage version != 0.
    //
    // <https://github.com/paritytech/substrate/pull/14641>
    let current_storage_version = <Pallet<T>>::current_storage_version();
    let onchain_storage_version = <Pallet<T>>::on_chain_storage_version();

    weight.saturating_accrue(T::DbWeight::get().reads(1));

    if onchain_storage_version == 0 && current_storage_version != 0 {
        // Set new storage version.
        current_storage_version.put::<Pallet<T>>();

        // Write the onchain storage version.
        weight = weight.saturating_add(T::DbWeight::get().writes(1));
    }

    weight
}

/// Check the state before the bridges initialization.
///
/// Panics if anything goes wrong.
#[cfg(feature = "try-runtime")]
pub fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
    // Do nothing.
    Ok(Vec::new())
}

/// Check the state after the bridges initialization.
///
/// Panics if anything goes wrong.
#[cfg(feature = "try-runtime")]
pub fn post_upgrade<T: Config>(_state: Vec<u8>) -> Result<(), &'static str> {
    use frame_support::{storage_root, StateVersion};

    let storage_root_before = storage_root(StateVersion::V1);

    if !Pallet::<T>::is_balanced()? {
        return Err("currencies are not balanced");
    }

    assert_eq!(storage_root_before, storage_root(StateVersion::V1));

    assert_eq!(
        <Pallet<T>>::on_chain_storage_version(),
        <Pallet<T>>::current_storage_version()
    );

    Ok(())
}
