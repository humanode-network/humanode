//! The version definition of the peer's related components.

use serde::Serialize;

/// Define API versions.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiVersions {
    /// Bioauth flow version.
    pub bioauth_flow: u8,
}

/// The Current API versions.
pub const API_VERSIONS: ApiVersions = ApiVersions { bioauth_flow: 2 };
