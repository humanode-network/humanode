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

/// The additional data about the session that FaceTec communicates back to us
/// with each response.
#[derive(Debug)]
pub struct AdditionalSessionData {
    // "isAdditionalDataPartiallyIncomplete": false,
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
#[derive(Debug)]
pub struct FaceScanSecurityChecks {
    // "auditTrailVerificationCheckSucceeded": true,
// "faceScanLivenessCheckSucceeded": false,
// "replayCheckSucceeded": true,
// "sessionTokenCheckSucceeded": true
}

/// The call data that FaceTec includes with each response.
#[derive(Debug)]
pub struct CallData {
    // "tid": "AAAAAAAAAAA-00000000-0000-0000-0000-000000000000",
//                 "path": "/enrollment-3d",
//                 "date": "Jan 01, 2000 00:00:00 AM",
//                 "epochSecond": 946684800,
//                 "requestMethod": "POST"
}

/// The server info that FaceTec sends us with each response.
#[derive(Debug)]
pub struct ServerInfo {
    // "version": "9.0.5",
// "mode": "Development Only",
// "notice": "Notice"
}
