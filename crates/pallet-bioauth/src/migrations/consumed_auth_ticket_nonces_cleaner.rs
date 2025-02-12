//! Migration to clean consumed auth ticket nonces.

#[cfg(feature = "try-runtime")]
use frame_support::sp_std::vec::Vec;
use frame_support::{log::info, pallet_prelude::*, traits::OnRuntimeUpgrade};

use crate::{Config, Pallet};

/// Execute migration to clean consumed auth ticket nonces.
pub struct ConsumedAuthTicketNoncesCleaner<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for ConsumedAuthTicketNoncesCleaner<T> {
    fn on_runtime_upgrade() -> Weight {
        let pallet_name = Pallet::<T>::name();
        info!("{pallet_name}: Running migration to clean consumed auth ticket nonces");

        todo!()
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        // Do nothing.
        Ok(Vec::new())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
        todo!()
    }
}
