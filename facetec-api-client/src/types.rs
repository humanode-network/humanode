//! Common types.

use serde::Deserialize;

/// A type that represents an opaque Base64 data.
///
/// Opaque in a sense that our code does not try to validate or decode it.
/// We could decode the opaque Base64 representation, and then reencode it,
/// but since we're just passing this value through - we can leave it as is,
/// and we don't really have to do anything with it.
pub type OpaqueBase64DataRef<'a> = &'a str;

/// The type to be used everywhere as the match level.
pub type MatchLevel = i64;

/// The additional data about the session that FaceTec communicates back to us
/// with each response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdditionalSessionData {
    /// TODO: document.
    pub is_additional_data_partially_incomplete: bool,
    // "platform": "android",
    // "appID": "com.facetec.sampleapp",
    // "installationID": "0000000000000000",
    // "deviceModel": "Pixel 4",
    // "deviceSDKVersion": "9.0.2",
    // "sessionID": "00000000-0000-0000-0000-000000000000",
    // "userAgent": "UserAgent",
    // "ipAddress": "1.2.3.4"
}

/// The report on the security checks.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FaceScanSecurityChecks {
    /// TODO: document
    audit_trail_verification_check_succeeded: bool,
    /// TODO: document
    face_scan_liveness_check_succeeded: bool,
    /// TODO: document
    replay_check_succeeded: bool,
    /// TODO: document
    session_token_check_succeeded: bool,
}

/// The call data that FaceTec includes with each response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallData {
    /// Some opaque transaction identifier.
    tid: String,
    /// Request URI path.
    path: String,
    /// Request date, as a string in the US locale, without timezone or offset.
    date: String,
    /// The unix-time representation of the request date.
    epoch_second: i64,
    /// The HTTP method the request was issued with.
    request_method: String,
}

/// The server info that FaceTec sends us with each response.
#[derive(Debug, Deserialize)]
pub struct ServerInfo {
    /// Version of the server.
    pub version: String,
    /// Mode of the operation of the server.
    pub mode: String,
    /// A notice that server gives with this response.
    pub notice: String,
}

/// A common FaceTec API response portion.
#[derive(Debug, Deserialize)]
pub struct CommonResponse {
    /// The additional session information included in this response.
    #[serde(rename = "additionalSessionData")]
    pub additional_session_data: AdditionalSessionData,
    /// The information about the API call the request was to.
    #[serde(rename = "callData")]
    pub call_data: CallData,
    /// The information about the server.
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

/// A FaceScan-related FaceTec API response portion.
#[derive(Debug, Deserialize)]
pub struct FaceScanResponse {
    /// The the information about the security checks over the FaceScan data.
    #[serde(rename = "faceScanSecurityChecks")]
    pub face_scan_security_checks: FaceScanSecurityChecks,
    /// Something to do with the retry screen of the FaceTec Device SDK.
    /// TODO: find more info on this parameter.
    #[serde(rename = "faceTecRetryScreen")]
    pub face_tec_retry_screen: i64,
    /// Something to do with the retry screen of the FaceTec Device SDK.
    /// TODO: find more info on this parameter.
    #[serde(rename = "retryScreenEnumInt")]
    pub retry_screen_enum_int: i64,
    /// The age group enum id that the input FaceScan was classified to.
    #[serde(rename = "ageEstimateGroupEnumInt")]
    pub age_estimate_group_enum_int: i64,
}
