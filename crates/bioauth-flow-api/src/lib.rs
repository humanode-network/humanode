//! The runtime API for the bioauth flow.
//!
//! Intended for powering the bioauth flow UI with the up-to-date information regarding
//! the bioauth state of the node from the chain.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::prelude::*;

/// The bioauth status.
#[derive(Debug, PartialEq, Eq, Decode, Encode, TypeInfo)]
pub enum BioauthStatus<Timestamp> {
    /// No active authentication is present.
    Inactive,
    /// An active authentication exists.
    Active {
        /// The timestamp when the authentication will expire.
        expires_at: Timestamp,
    },
}

sp_api::decl_runtime_apis! {
    /// Runtime API for the bioauth flow.
    pub trait BioauthFlowApi<Id, Timestamp: Decode> where Id: Encode {
        /// Determine the bioauth status for the given `id` at the current block.
        ///
        /// This call is intended for use in the bioauth flow, and the `id` passed is likely.
        fn bioauth_status(id: &Id) -> BioauthStatus<Timestamp>;

        /// Create an extrinsic for submitting auth ticket.
        fn create_authenticate_extrinsic(
            auth_ticket: Vec<u8>,
            auth_ticket_signature: Vec<u8>
        ) -> Block::Extrinsic;
    }
}
