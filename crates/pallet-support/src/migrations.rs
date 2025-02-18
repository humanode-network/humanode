//! Migrations related support code.

#[cfg(feature = "try-runtime")]
use frame_support::sp_std::vec::Vec;
use frame_support::{
    log,
    pallet_prelude::*,
    sp_io::{hashing::twox_128, storage::clear_prefix, KillStorageResult},
    weights::RuntimeDbWeight,
};

/// `RemovePallet` is a utility struct used to remove all storage items associated with a specific
/// pallet.
///
/// The implementation is based on [`frame_support::migrations::RemovePallet`], but doesn't focus
/// just on removing all storage items per one runtime upgrade in one block. Additionally, it allows
/// to use capabilities of `clear_prefix` with limit that can be used to partially delete a prefix
/// storage in case it is too large to delete in one block.
///
/// We are going to upstream improved logic to <https://github.com/paritytech/polkadot-sdk> when
/// we switched our substrate dependencies from archived substrate fork to polkadot-sdk fork.
pub struct RemovePallet<
    P: Get<&'static str>,
    Limit: Get<Option<u32>>,
    DbWeight: Get<RuntimeDbWeight>,
>(PhantomData<(P, Limit, DbWeight)>);

impl<P: Get<&'static str>, Limit: Get<Option<u32>>, DbWeight: Get<RuntimeDbWeight>>
    frame_support::traits::OnRuntimeUpgrade for RemovePallet<P, Limit, DbWeight>
{
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        let pallet_name = P::get();
        let hashed_prefix = twox_128(P::get().as_bytes());

        let keys_removed: u64 = match clear_prefix(&hashed_prefix, Limit::get()) {
            KillStorageResult::AllRemoved(value) => {
                log::info!("{pallet_name}: Removed all {value} keys 🧹");
                value
            }
            KillStorageResult::SomeRemaining(value) => {
                log::info!("{pallet_name}: Removed {value} keys, some of them still remain 🧹");
                value
            }
        }
        .into();

        DbWeight::get().reads_writes(keys_removed, keys_removed)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        // Do nothing.
        Ok(Vec::new())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
        // Do nothing.
        Ok(())
    }
}
