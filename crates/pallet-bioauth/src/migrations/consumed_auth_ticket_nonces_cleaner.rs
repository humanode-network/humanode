//! Migration to clean consumed auth ticket nonces.

#[cfg(feature = "try-runtime")]
use frame_support::sp_std::vec::Vec;
use frame_support::{log::info, pallet_prelude::*, traits::OnRuntimeUpgrade};

use crate::{Config, ConsumedAuthTicketNonces, Pallet};

/// Execute migration to clean consumed auth ticket nonces except ones related to current generation.
pub struct ConsumedAuthTicketNoncesCleaner<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for ConsumedAuthTicketNoncesCleaner<T> {
    fn on_runtime_upgrade() -> Weight {
        let pallet_name = Pallet::<T>::name();
        info!("{pallet_name}: Running migration to clean consumed auth ticket nonces");

        ConsumedAuthTicketNonces::<T>::mutate(|consumed_auth_ticket_nonces| {
            if let Some(last_consumed_auth_ticket_nonce) = consumed_auth_ticket_nonces.last() {
                // Current generation is represented by first 16 bytes at nonce.
                let current_generation = last_consumed_auth_ticket_nonce[..16].to_vec();

                consumed_auth_ticket_nonces.retain(|consumed_auth_ticket_nonce| {
                    consumed_auth_ticket_nonce.starts_with(&current_generation)
                });
            }
        });

        info!("{pallet_name}: Migrated");

        T::DbWeight::get().reads_writes(1, 1)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        let pre_upgrade_state = match ConsumedAuthTicketNonces::<T>::get().last() {
            Some(last_consumed_auth_ticket_nonce) => {
                last_consumed_auth_ticket_nonce.clone()[..16].to_vec()
            }
            None => Vec::new(),
        };

        Ok(pre_upgrade_state)
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
        if state.is_empty() {
            return Ok(());
        }

        let consumed_auth_ticket_nonces = ConsumedAuthTicketNonces::<T>::get();
        for consumed_auth_ticket_nonce in consumed_auth_ticket_nonces {
            ensure!(
                consumed_auth_ticket_nonce.starts_with(&state),
                "Consumed auth ticket nonce should belong to current generation",
            );
        }

        Ok(())
    }
}
