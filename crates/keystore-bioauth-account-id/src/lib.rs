//! Crypto primitives for [`KeystoreBioauthAccountId`] key type.

// Either generate code at stadard mode, or `no_std`, based on the `std` feature presence.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_application_crypto::KeyTypeId;

/// Keystore Bioauth Account ID key type definition.
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"kbai");

/// App key definition.
mod app {
    use sp_application_crypto::{app_crypto, sr25519};
    app_crypto!(sr25519, super::KEY_TYPE);
}

/// App key export.
pub type KeystoreBioauthAccountId = app::Public;
