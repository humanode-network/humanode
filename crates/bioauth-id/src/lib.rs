//! Crypto primitives for bioauth account.

/// Bioauth crypto keys definition.
mod app {
    use sp_application_crypto::{app_crypto, key_types::ACCOUNT, sr25519};
    app_crypto!(sr25519, ACCOUNT);
}

/// A Bioauth authority identifier.
pub type AuthorityId = app::Public;
