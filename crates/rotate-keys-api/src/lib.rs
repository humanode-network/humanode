//! The runtime API for the session keys rotation.

#![cfg_attr(not(feature = "std"), no_std)]

sp_api::decl_runtime_apis! {
    /// Runtime API for the session keys rotation.
    pub trait RotateKeysApi {

        /// Rotate session keys.
        fn rotate_session_keys();
    }
}
