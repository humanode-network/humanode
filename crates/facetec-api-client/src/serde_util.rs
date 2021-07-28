//! Utilities for serde.

use serde::Deserialize;

/// Internal type to parse values on the contents.
/// Useful for extracting errors from 200-ok responses.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Either<A, B> {
    /// Left variant.
    Left(A),
    /// Right variant.
    Right(B),
}
