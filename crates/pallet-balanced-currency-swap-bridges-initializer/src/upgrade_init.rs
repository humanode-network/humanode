//! Initialization of the bridge pot accounts on runtime upgrade.

use frame_support::pallet_prelude::*;
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

use crate::{Config, InitializerVersion, Pallet, CURRENT_BRIDGES_INITIALIZER_VERSION};

/// Initialize the bridges pot accounts.
pub fn on_runtime_upgrade<T: Config>() -> Weight {
    let initializer_version = <InitializerVersion<T>>::get();
    let mut weight = T::DbWeight::get().reads(1);

    if initializer_version != CURRENT_BRIDGES_INITIALIZER_VERSION {
        let is_balanced = Pallet::<T>::is_balanced().unwrap_or_default();
        weight += T::DbWeight::get().reads(8);

        if !is_balanced {
            match Pallet::<T>::initialize() {
                Ok(w) => weight += w,
                Err(err) => sp_tracing::error!("error during bridges initialization: {err:?}"),
            }
        }

        <InitializerVersion<T>>::put(CURRENT_BRIDGES_INITIALIZER_VERSION);
    }

    weight
}

/// Check the state before the bridges initialization.
///
/// Panics if anything goes wrong.
#[cfg(feature = "try-runtime")]
pub fn pre_upgrade<T: Config>() -> Result<Vec<u8>, &'static str> {
    // Do nothing.
    Ok(Vec::new())
}

/// Check the state after the bridges initialization.
///
/// Panics if anything goes wrong.
#[cfg(feature = "try-runtime")]
pub fn post_upgrade<T: Config>(_state: Vec<u8>) -> Result<(), &'static str> {
    if !Pallet::<T>::is_balanced()? {
        return Err("currencies are not balanced");
    }

    Ok(())
}
