//! Common logic parameters.

/// This is the error message that FaceTec server returns when it
/// encounters an `externalDatabaseRefID` that is already in use.
/// For the lack of a better option, we have to compare the error messages,
/// which is not a good idea, and there should've been a better way.
pub const EXTERNAL_DATABASE_REF_ID_ALREADY_IN_USE_ERROR_MESSAGE: &str =
    "An enrollment already exists for this externalDatabaseRefID.";

/// The group name at 3D DB.
pub const DB_GROUP_NAME: &str = "humanode";
/// The match level to use throughout the code.
pub const MATCH_LEVEL: i64 = 10;
