//! Initialization of the bridge pot accounts on runtime upgrade.

use frame_support::pallet_prelude::*;

use crate::Pallet;

/// Initialize the bridges pot accounts.
pub fn on_runtime_upgrade<T: crate::Config>() -> Weight {
    let is_balanced = Pallet::<T>::is_balanced().unwrap_or_default();
    let mut weight = T::DbWeight::get().reads(8);

    if !is_balanced {
        match Pallet::<T>::initialize() {
            Ok(w) => weight += w,
            Err(err) => sp_tracing::error!("error during bridges initialization: {err:?}"),
        }
    }

    weight
}

/// Check the state before the bridges initialization.
///
/// Panics if anything goes wrong.
#[cfg(feature = "try-runtime")]
pub fn pre_upgrade<T: Config<I>, I: 'static>() -> Result<Vec<u8>, &'static str> {
    // do nothing
}

/// Check the state after the bridges initialization.
///
/// Panics if anything goes wrong.
#[cfg(feature = "try-runtime")]
pub fn post_upgrade<T: Config<I>, I: 'static>(state: Vec<u8>) -> Result<(), &'static str> {
    if !Pallet::<T>::is_balanced() {
        return Err("currencies are not balanced");
    }

    Ok(())
}
