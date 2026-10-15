//! The step context related error kinds.

/// The step context related error kinds.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The context did not decode.
    #[error("invalid step context: {0}")]
    Decode(codec::Error),

    /// The nonce can not be incremented.
    #[error("invalid step context: nonce overflow")]
    NonceOverflow,
}
