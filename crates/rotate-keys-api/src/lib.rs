//! The runtime API for the session keys rotation.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
    /// Runtime API for the session keys rotation.
    pub trait RotateKeysApi<Id: Encode> {

        /// Rotate session keys.
        fn rotate_session_keys(id: &Id, session_keys: Vec<u8>) -> Block::Extrinsic;
    }
}
