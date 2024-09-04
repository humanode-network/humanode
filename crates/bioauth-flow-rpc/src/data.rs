//! The RPC request and response data, excluding errors.

/// The parameters necessary to initialize the FaceTec Device SDK.
pub type FacetecDeviceSdkParams = serde_json::Map<String, serde_json::Value>;

/// The bioauth status as used in the RPC.
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BioauthStatus<Timestamp> {
    /// When the status can't be determined, but there was no error.
    /// Can happen if the validator key is absent.
    Unknown,
    /// There is no active authentication for the currently used validator key.
    Inactive,
    /// There is an active authentication for the currently used validator key.
    Active {
        /// The timestamp when the authentication will expire.
        expires_at: Timestamp,
    },
}

impl<T> From<bioauth_flow_api::BioauthStatus<T>> for BioauthStatus<T> {
    fn from(status: bioauth_flow_api::BioauthStatus<T>) -> Self {
        match status {
            bioauth_flow_api::BioauthStatus::Inactive => Self::Inactive,
            bioauth_flow_api::BioauthStatus::Active { expires_at } => Self::Active { expires_at },
        }
    }
}

/// `enroll_v2` flow result.
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrollV2Result {
    /// Scan result blob.
    pub scan_result_blob: Option<String>,
}

/// `authenticate_v2` related flow result.
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateV2Result {
    /// An opaque auth ticket generated for this authentication attempt.
    pub auth_ticket: Box<[u8]>,
    /// The robonode signature for this opaque auth ticket.
    pub auth_ticket_signature: Box<[u8]>,
    /// Scan result blob.
    pub scan_result_blob: Option<String>,
}
