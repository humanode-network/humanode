//! Build envrionment information.

/// The git sha of the code during the build.
pub const GIT_SHA: &str = env!("VERGEN_GIT_SHA");

/// The cargo profile used for the build.
pub const CARGO_PROFILE: &str = env!("VERGEN_CARGO_PROFILE");

/// The cargo features activated during the build.
pub const CARGO_FEATURES: &str = env!("VERGEN_CARGO_FEATURES");
