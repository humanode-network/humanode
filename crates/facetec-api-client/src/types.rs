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

/// The report on the security checks.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FaceScanSecurityChecks {
    /// TODO: document
    pub audit_trail_verification_check_succeeded: bool,
    /// TODO: document
    pub face_scan_liveness_check_succeeded: bool,
    /// TODO: document
    pub replay_check_succeeded: bool,
    /// TODO: document
    pub session_token_check_succeeded: bool,
}

impl FaceScanSecurityChecks {
    /// Returns `true` only if all of the underlying checks are `true`.
    pub fn all_checks_succeeded(&self) -> bool {
        self.audit_trail_verification_check_succeeded
            && self.face_scan_liveness_check_succeeded
            && self.replay_check_succeeded
            && self.session_token_check_succeeded
    }
}

/// A FaceScan-related FaceTec API response portion.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FaceScanResponse {
    /// The the information about the security checks over the FaceScan data.
    pub face_scan_security_checks: FaceScanSecurityChecks,
    /// Something to do with the retry screen of the FaceTec Device SDK.
    /// TODO: find more info on this parameter.
    pub retry_screen_enum_int: i64,
    /// The age group enum id that the input FaceScan was classified to.
    pub age_estimate_group_enum_int: i64,
}
