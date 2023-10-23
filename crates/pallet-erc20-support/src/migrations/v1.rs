//! Migration to Version 1.

use frame_support::{dispatch::GetStorageVersion, sp_tracing::info, traits::Get, weights::Weight};
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

use crate::BalanceOf;
use crate::{Approvals, Config, Pallet};

/// Migrate from version 0 to 1.
pub fn migrate<T: Config<I>, I: 'static>() -> Weight {
    let current = <Pallet<T, I>>::current_storage_version();
    let onchain = <Pallet<T, I>>::on_chain_storage_version();

    // Read the onchain version.
    let mut weight: Weight = T::DbWeight::get().reads(1);

    info!(message = "Running migration to v1", from = ?onchain);

    if onchain == 1 {
        info!(message = "Already at version 1, nothing to do");
        return weight;
    }

    <Approvals<T, I>>::translate(|_owner, _spender, amount: BalanceOf<T, I>| Some(amount.into()));

    // Set new version.
    current.put::<Pallet<T, I>>();

    // Write the onchain version.
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    // Done.
    weight
}

/// Check the state before the migration.
///
/// Panics if anything goes wrong.
#[cfg(feature = "try-runtime")]
pub fn pre_migrate<T: Config<I>, I: 'static>() -> Vec<u8> {
    sp_std::vec![]
}

/// Check the state after the migration.
///
/// Panics if anything goes wrong.
#[cfg(feature = "try-runtime")]
pub fn post_migrate<T: Config<I>, I: 'static>(_state: Vec<u8>) {
    // Ensure version is updated correctly.
    let onchain = <Pallet<T, I>>::on_chain_storage_version();
    assert_eq!(onchain, 1);
}
