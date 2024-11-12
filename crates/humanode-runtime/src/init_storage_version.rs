use core::marker::PhantomData;

use frame_support::{
    log::info,
    traits::{Get, GetStorageVersion, OnRuntimeUpgrade, PalletInfoAccess},
    weights::Weight,
};
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

/// Pallet storage version initializer.
pub struct InitStorageVersion<P, R>(PhantomData<(P, R)>);

impl<P, R> OnRuntimeUpgrade for InitStorageVersion<P, R>
where
    P: GetStorageVersion + PalletInfoAccess,
    R: frame_system::Config,
{
    fn on_runtime_upgrade() -> Weight {
        // Properly manage default on chain storage version as the pallet was added after genesis
        // with initial storage version != 0.
        //
        // <https://github.com/paritytech/substrate/pull/14641>
        let current_storage_version = P::current_storage_version();
        let onchain_storage_version = P::on_chain_storage_version();

        let mut weight = R::DbWeight::get().reads(1);

        if onchain_storage_version == 0 && current_storage_version != 0 {
            info!(
                "{}: Initializing an unset on-chain storage version to {:?}, assuming the effective state version is the latest pallet version",
                P::name(),
                current_storage_version,
            );

            // Set new storage version.
            current_storage_version.put::<P>();

            // Write the onchain storage version.
            weight = weight.saturating_add(R::DbWeight::get().writes(1));
        } else {
            info!(
                "{}: Nothing to do. This runtime upgrade probably should be removed.",
                P::name(),
            );
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
        assert_eq!(P::on_chain_storage_version(), P::current_storage_version());
        Ok(())
    }
}
