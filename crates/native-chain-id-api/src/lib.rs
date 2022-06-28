//! The runtime API for the author extension logic.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
    /// Runtime API for the author extension logic.
    pub trait NativeChainIdApi {
        /// Create signed set_keys extrinsic.
        fn get() -> u16;
    }
}
