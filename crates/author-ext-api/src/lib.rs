//! The runtime API for the signed extrinsics creation.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
    /// Runtime API for the signed extrinsics creation.
    pub trait AuthorExtApi<Id: Encode> {

        /// Create signed set_keys extrinsic.
        fn create_signed_set_keys_extrinsic(id: &Id, session_keys: Vec<u8>) -> Option<Block::Extrinsic>;
    }
}
