//! The runtime API for the bioauth flow.
//!
//! Intended for powering the bioauth flow UI with the up-to-date information regarding
//! the bioauth state of the node from the chain.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
    /// Runtime API for extracting BioauthId associated to the provided ValidatorId.
    pub trait BioauthIdApi<ValidatorId: Encode, BioauthId: Decode> {
        /// Extract the corresponding BioauthId.
        fn extract_bioauth_id(validator_id: &ValidatorId) -> BioauthId;
    }
}
