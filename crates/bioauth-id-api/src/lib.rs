//! The runtime API for the bioauth id related logic.

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
