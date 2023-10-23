//! Migration to Version 1.

use frame_support::{sp_tracing::info, traits::Get, weights::Weight};
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

use crate::BalanceOf;
use crate::{Approvals, Config};

/// Migrate from version 0 to 1.
pub fn migrate<T: Config<I>, I: 'static>() -> Weight {
    info!("Running migration to v1");

    let mut weight: Weight = T::DbWeight::get().reads(0);

    <Approvals<T, I>>::translate(|_owner, _spender, amount: BalanceOf<T, I>| {
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
        Some(amount.into())
    });

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
    let storage_version = StorageVersion::get::<Pallet<T, I>>();
    assert_eq!(storage_version, 1);
}
