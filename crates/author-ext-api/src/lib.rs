//! The runtime API for the author extension logic.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::{prelude::string::String, TypeInfo};
use sp_std::prelude::*;

/// An error that can occur during signed `set_keys` extrinsic creation.
#[derive(Debug, PartialEq, Eq, Decode, Encode, TypeInfo)]
pub enum CreateSignedSetKeysExtrinsicError {
    /// Unable to decode session keys.
    SessionKeysDecoding(String),
    /// Unable to create signed `set_keys` extrinsic.
    SignedExtrinsicCreation,
}

sp_api::decl_runtime_apis! {
    /// Runtime API for the author extension logic.
    pub trait AuthorExtApi<Id> where Id: Encode {
        /// Create signed set_keys extrinsic.
        fn create_signed_set_keys_extrinsic(id: &Id, session_keys: Vec<u8>) -> Result<Block::Extrinsic, CreateSignedSetKeysExtrinsicError>;
    }
}
