//! Common types.

/// A type that represents an opaque Base64 data.
///
/// Opaque in a sense that our code does not try to validate or decode it.
/// We could decode the opaque Base64 representation, and then reencode it,
/// but since we're just passing this value through - we can leave it as is,
/// and we don't really have to do anything with it.
pub type OpaqueBase64DataRef<'a> = &'a str;

/// The type to be used everywhere as the match level.
pub type MatchLevel = i64;
