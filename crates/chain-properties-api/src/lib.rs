//! The runtime API for getting chain properties.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
    /// Runtime API for getting chain properties.
    pub trait ChainPropertiesApi {
        /// Get ss58 prefix.
        fn ss58_prefix() -> u16;
    }
}
