//! Migration to Version 1 from Version 0.

#[cfg(feature = "try-runtime")]
use frame_support::sp_std::{vec, vec::Vec};
use frame_support::{
    log::info,
    pallet_prelude::*,
    sp_std,
    traits::{Get, OnRuntimeUpgrade},
    weights::Weight,
};

use crate::{Approvals, BalanceOf, Config, Pallet};

/// Execute migration to Version 1 from Version 0.
pub struct MigrationV0ToV1<T, I = ()>(sp_std::marker::PhantomData<(T, I)>);

impl<T: Config<I>, I: 'static> OnRuntimeUpgrade for MigrationV0ToV1<T, I> {
    fn on_runtime_upgrade() -> Weight {
        migrate::<T, I>()
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        Ok(vec![])
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
        ensure!(
            <Pallet<T, I>>::on_chain_storage_version() == <Pallet<T, I>>::current_storage_version(),
            "the current storage version and onchain storage version should be the same"
        );
        Ok(())
    }
}

/// Migrate from version 0 to 1.
fn migrate<T: Config<I>, I: 'static>() -> Weight {
    let onchain_version = Pallet::<T, I>::on_chain_storage_version();

    // Read the onchain version.
    let mut weight: Weight = T::DbWeight::get().reads(1);

    if onchain_version != 0 {
        info!("Already not at version 0, nothing to do. This migrarion probably should be removed");
        return weight;
    }

    info!("Running migration to v1");

    <Approvals<T, I>>::translate(|_owner, _spender, amount: BalanceOf<T, I>| {
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
        Some(amount.into())
    });

    // Set storage version to `1`.
    StorageVersion::new(1).put::<Pallet<T, I>>();
    weight.saturating_accrue(T::DbWeight::get().writes(1));

    info!("Migrated to v1");

    weight
}
