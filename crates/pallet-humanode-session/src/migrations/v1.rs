//! Migration to Version 1.

#[cfg(feature = "try-runtime")]
use frame_support::sp_std::{vec, vec::Vec};
use frame_support::{
    log::info,
    pallet_prelude::*,
    sp_std, storage_alias,
    traits::{Get, OnRuntimeUpgrade},
    weights::Weight,
};

use crate::{Config, CurrentSessionIndex, IdentificationFor, Pallet, SessionIdentities};

/// Execute migration to Version 1.
pub struct MigrationToV1<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrationToV1<T> {
    fn on_runtime_upgrade() -> Weight {
        migrate::<T>()
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        Ok(pre_migrate::<T>())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
        post_migrate::<T>(state);
        Ok(())
    }
}

/// The Version 0 identities storage.
#[storage_alias]
pub type CurrentSessionIdentities<T: Config> = StorageMap<
    Pallet<T>,
    Twox64Concat,
    <T as frame_system::Config>::AccountId,
    IdentificationFor<T>,
    OptionQuery,
>;

/// Migrate from version 0 to 1.
pub fn migrate<T: Config>() -> Weight {
    let onchain = <Pallet<T>>::on_chain_storage_version();

    // Read the onchain version.
    let mut weight: Weight = T::DbWeight::get().reads(1);

    if onchain != 0 {
        info!("Already not at version 0, nothing to do. This migrarion probably should be removed");
        return weight;
    }

    info!("Running migration to v1");

    // Restore session index from the session pallet.
    let session_index = <pallet_session::Pallet<T>>::current_index();
    <CurrentSessionIndex<T>>::put(session_index);
    // Read the session index from the session pallet, then write it to our own state.
    weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

    // Move the old identities to the new ones, but one session forward.
    <CurrentSessionIdentities<T>>::translate(|key, old: IdentificationFor<T>| {
        // u32 is big enough for this overflow to be practically impossible.
        <SessionIdentities<T>>::insert(session_index.checked_add(1).unwrap(), key, old);
        // Read the old value, insert one new value, and drop the old one.
        weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 2));
        None
    });

    // Set storage version to `1`.
    StorageVersion::new(1).put::<Pallet<T>>();
    weight.saturating_accrue(T::DbWeight::get().writes(1));

    // Done.
    weight
}

/// Check the state before the migration.
///
/// Panics if anything goes wrong.
#[cfg(feature = "try-runtime")]
pub fn pre_migrate<T: Config>() -> Vec<u8> {
    let onchain = <Pallet<T>>::on_chain_storage_version();

    // Disable the check for newer versions by returning an empty state.
    if onchain >= 1 {
        return vec![];
    }

    // Ensure the new identities don't exist yet (i.e. we have clear space to migrate).
    assert_eq!(<SessionIdentities<T>>::iter().next(), None);

    // Record the count of identities.
    let identities_count: u64 = <CurrentSessionIdentities<T>>::iter()
        .count()
        .try_into()
        .unwrap();
    identities_count.encode()
}

/// Check the state after the migration.
///
/// Panics if anything goes wrong.
#[cfg(feature = "try-runtime")]
pub fn post_migrate<T: Config>(state: Vec<u8>) {
    // Empty state means that the check is disabled.
    if state.is_empty() {
        return;
    }

    // Ensure version is updated correctly.
    let onchain = <Pallet<T>>::on_chain_storage_version();
    assert_eq!(onchain, 1);

    // Ensure the old identities are cleared.
    assert_eq!(<CurrentSessionIdentities<T>>::iter().next(), None);

    // Ensure the identities count matches.
    let new_identities_count: u64 = <SessionIdentities<T>>::iter().count().try_into().unwrap();
    let old_identities_count: u64 = codec::Decode::decode(&mut &*state).unwrap();
    assert_eq!(new_identities_count, old_identities_count);
}
