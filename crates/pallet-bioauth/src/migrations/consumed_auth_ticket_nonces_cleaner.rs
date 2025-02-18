//! Migration to clean consumed auth ticket nonces except ones related to current generation.

#[cfg(feature = "try-runtime")]
use frame_support::sp_std::vec::Vec;
use frame_support::{log::info, pallet_prelude::*, traits::OnRuntimeUpgrade};

use crate::{Config, ConsumedAuthTicketNonces, Pallet};

/// Execute migration to clean consumed auth ticket nonces except ones related to current generation.
pub struct ConsumedAuthTicketNoncesCleaner<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for ConsumedAuthTicketNoncesCleaner<T> {
    fn on_runtime_upgrade() -> Weight {
        let pallet_name = Pallet::<T>::name();
        info!("{pallet_name}: Running migration to clean consumed auth ticket nonces except ones related to current generation");

        ConsumedAuthTicketNonces::<T>::mutate(|consumed_auth_ticket_nonces| {
            if let Some(last_consumed_auth_ticket_nonce) = consumed_auth_ticket_nonces.last() {
                let consumed_auth_ticket_nonces_number_before = consumed_auth_ticket_nonces.len();

                // Current generation is represented by first 16 bytes at nonce.
                let current_generation = last_consumed_auth_ticket_nonce[..16].to_vec();

                consumed_auth_ticket_nonces.retain(|consumed_auth_ticket_nonce| {
                    consumed_auth_ticket_nonce.starts_with(&current_generation)
                });

                let removed_number = consumed_auth_ticket_nonces_number_before
                    .checked_sub(consumed_auth_ticket_nonces.len())
                    .expect("valid operation; qed");

                info!("{pallet_name}: Removed {removed_number} consumed auth ticket nonces");
            } else {
                info!("{pallet_name}: Nothing to remove");
            }
        });

        T::DbWeight::get().reads_writes(1, 1)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        // Record the last consumed auth ticket nonce, otherwise return an empty result
        // if there are no nonces yet.
        let pre_upgrade_state = match ConsumedAuthTicketNonces::<T>::get().last() {
            Some(last_consumed_auth_ticket_nonce) => last_consumed_auth_ticket_nonce.to_vec(),
            None => Vec::new(),
        };

        Ok(pre_upgrade_state)
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
        // Empty state means there are no consumed auth ticket nonces.
        if state.is_empty() {
            return Ok(());
        }

        let last_consumed_auth_ticket_nonce = state;
        let current_generation = &last_consumed_auth_ticket_nonce[..16];

        let consumed_auth_ticket_nonces = ConsumedAuthTicketNonces::<T>::get();

        ensure!(
            last_consumed_auth_ticket_nonce == consumed_auth_ticket_nonces.last().unwrap().to_vec(),
            "The last record we selected earlier should remain in it's position of being last"
        );

        for consumed_auth_ticket_nonce in consumed_auth_ticket_nonces {
            ensure!(
                consumed_auth_ticket_nonce.starts_with(current_generation),
                "Consumed auth ticket nonce should belong to current generation",
            );
        }

        Ok(())
    }
}
