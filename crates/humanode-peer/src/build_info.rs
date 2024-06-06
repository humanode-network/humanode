//! Build environment information.

/// The git sha of the code during the build.
pub const GIT_SHA: &str = env!("VERGEN_GIT_SHA");

/// Whether the cargo debug profile (in contrary to a release profile) was used for the build.
pub const CARGO_DEBUG: &str = env!("VERGEN_CARGO_DEBUG");

/// The cargo features activated during the build.
pub const CARGO_FEATURES: &str = env!("VERGEN_CARGO_FEATURES");
