//! Crypto primitives for AccountId key type.

// Either generate code at stadard mode, or `no_std`, based on the `std` feature presence.
#![cfg_attr(not(feature = "std"), no_std)]

/// AccountId key type definition.
mod app {
    use sp_application_crypto::{app_crypto, key_types::ACCOUNT, sr25519};
    app_crypto!(sr25519, ACCOUNT);
}

/// AccountId key type identifier.
pub type KeystoreBioauthAccountId = app::Public;
