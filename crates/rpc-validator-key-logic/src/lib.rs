//! The validator key rpc related logic.

use bioauth_keys::traits::KeyExtractor as KeyExtractorT;

pub mod error_data;
mod errors;

pub use errors::*;

/// Try to extract the validator key.
pub fn validator_public_key<VKE>(validator_key_extractor: &VKE) -> Result<VKE::PublicKeyType, Error>
where
    VKE: KeyExtractorT,
    VKE::Error: std::fmt::Debug,
{
    let validator_public_key = validator_key_extractor
        .extract_key()
        .map_err(|error| {
            tracing::error!(
                message = "Unable to extract own key at bioauth flow RPC",
                ?error
            );
            Error::ValidatorKeyExtraction
        })?
        .ok_or(Error::MissingValidatorKey)?;
    Ok(validator_public_key)
}
