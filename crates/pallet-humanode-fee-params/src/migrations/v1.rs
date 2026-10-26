//! The migration to the storage version 1.
//!
//! The storage version 0 means the pallet storage has not been populated: this is the case on
//! the chains that upgrade from a runtime where this pallet did not exist.
//! This migration enacts the provided fee multipliers on such chains, which allows carrying over
//! the fee levels those chains had before the upgrade.

use core::marker::PhantomData;

use frame_support::{
    log::info,
    pallet_prelude::*,
    traits::{OnRuntimeUpgrade, PalletInfoAccess, StorageVersion},
};
#[cfg(feature = "try-runtime")]
use frame_support::{sp_runtime::TryRuntimeError, sp_std::vec::Vec};
use sp_runtime::FixedU128;

use crate::{Config, EvmFeeMultiplier, NativeFeeMultiplier, Pallet};

/// The storage version this migration brings the pallet to.
const TARGET_STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

/// Enacts the fee multipliers on the chains where the pallet storage has not been populated yet.
///
/// `NativeMultiplier` and `EvmMultiplier` provide the multipliers to enact.
pub struct EnactFeeMultipliers<T, NativeMultiplier, EvmMultiplier>(
    PhantomData<(T, NativeMultiplier, EvmMultiplier)>,
);

impl<T, NativeMultiplier, EvmMultiplier> OnRuntimeUpgrade
    for EnactFeeMultipliers<T, NativeMultiplier, EvmMultiplier>
where
    T: Config,
    NativeMultiplier: Get<FixedU128>,
    EvmMultiplier: Get<FixedU128>,
{
    fn on_runtime_upgrade() -> Weight {
        let pallet_name = Pallet::<T>::name();
        let onchain_storage_version = Pallet::<T>::on_chain_storage_version();

        if onchain_storage_version != 0 {
            info!(
                "{pallet_name}: Nothing to do, the on-chain storage version is {onchain_storage_version:?}. This runtime upgrade probably should be removed."
            );
            return T::DbWeight::get().reads(1);
        }

        let native_fee_multiplier = NativeMultiplier::get();
        let evm_fee_multiplier = EvmMultiplier::get();

        info!(
            "{pallet_name}: Enacting the native fee multiplier of {native_fee_multiplier:?} and the EVM fee multiplier of {evm_fee_multiplier:?}"
        );

        <NativeFeeMultiplier<T>>::put(native_fee_multiplier);
        <EvmFeeMultiplier<T>>::put(evm_fee_multiplier);
        TARGET_STORAGE_VERSION.put::<Pallet<T>>();

        T::DbWeight::get().reads_writes(1, 3)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
        // Record the on-chain storage version so that we can tell whether the migration was
        // supposed to run.
        Ok(Pallet::<T>::on_chain_storage_version().encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
        let pre_upgrade_storage_version: StorageVersion = Decode::decode(&mut &state[..])
            .map_err(|_| "the pre-upgrade state should decode into the storage version")?;

        ensure!(
            Pallet::<T>::on_chain_storage_version() == TARGET_STORAGE_VERSION,
            "the on-chain storage version should be at the target one after the upgrade"
        );

        if pre_upgrade_storage_version == 0 {
            ensure!(
                <NativeFeeMultiplier<T>>::get() == NativeMultiplier::get(),
                "the native fee multiplier should be the enacted one"
            );
            ensure!(
                <EvmFeeMultiplier<T>>::get() == EvmMultiplier::get(),
                "the EVM fee multiplier should be the enacted one"
            );
        }

        Ok(())
    }
}
