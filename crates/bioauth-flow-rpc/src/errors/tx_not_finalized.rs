//! The transaction not finalized related error kinds.

/// The transaction not finalized related error kinds.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Transaction is no longer valid in the current state.
    #[error("transaction is no longer valid in the current state")]
    Invalid,
    /// Transaction has been dropped from the pool because of the limit.
    #[error("transaction has been dropped from the pool because of the limit")]
    Dropped,
    /// Transaction has been replaced in the pool, by another transaction
    /// that provides the same tags. (e.g. same (sender, nonce)).
    #[error("transaction has been replaced in the pool, by another transaction")]
    Usurped,
    /// The block this transaction was included in has been retracted.
    #[error("the block this transaction was included in has been retracted")]
    Retracted,
    /// Maximum number of finality watchers has been reached,
    /// old watchers are being removed.
    #[error("finality timeout")]
    FinalityTimeout,
}
