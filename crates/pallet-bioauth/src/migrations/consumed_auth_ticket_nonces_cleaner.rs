//! Migration to clean consumed auth ticket nonces.

#[cfg(feature = "try-runtime")]
use frame_support::sp_std::vec::Vec;
use frame_support::{log::info, pallet_prelude::*, traits::OnRuntimeUpgrade};

use crate::{Config, ConsumedAuthTicketNonces, Pallet};

/// Execute migration to clean consumed auth ticket nonces.
pub struct ConsumedAuthTicketNoncesCleaner<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for ConsumedAuthTicketNoncesCleaner<T> {
    fn on_runtime_upgrade() -> Weight {
        let pallet_name = Pallet::<T>::name();
        info!("{pallet_name}: Running migration to clean consumed auth ticket nonces");

        ConsumedAuthTicketNonces::<T>::mutate(|consumed_auth_ticket_nonces| {
            consumed_auth_ticket_nonces.clear();
        });

        info!("{pallet_name}: Migrated");

        T::DbWeight::get().reads_writes(1, 1)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        // Do nothing.
        Ok(Vec::new())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
        ensure!(
            ConsumedAuthTicketNonces::<T>::decode_len().unwrap() == 0,
            "Consumed auth ticket nonces should be empty",
        );

        Ok(())
    }
}
