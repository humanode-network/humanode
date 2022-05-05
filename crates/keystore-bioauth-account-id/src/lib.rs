//! Crypto primitives for AccountId key type.

// Either generate code at stadard mode, or `no_std`, based on the `std` feature presence.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::offchain::AppCrypto;
use sp_runtime::{MultiSignature, MultiSigner};

/// AccountId key type definition.
mod app {
    use sp_application_crypto::{app_crypto, key_types::ACCOUNT, sr25519};
    app_crypto!(sr25519, ACCOUNT);
}

/// AccountId key type identifier.
pub type KeystoreBioauthAccountId = app::Public;

impl AppCrypto<MultiSigner, MultiSignature> for KeystoreBioauthAccountId {
    type RuntimeAppPublic = KeystoreBioauthAccountId;
    type GenericSignature = sp_core::sr25519::Signature;
    type GenericPublic = sp_core::sr25519::Public;
}
