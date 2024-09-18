//! Custom rpc error codes.

/// Signer has failed.
pub const SIGN: i32 = 100;

/// Request to robonode has failed.
pub const ROBONODE: i32 = 200;

/// Call to runtime api has failed.
pub const RUNTIME_API: i32 = 300;

/// Authenticate transaction has failed.
pub const TRANSACTION: i32 = 400;

/// Validator key is not available.
pub const MISSING_VALIDATOR_KEY: i32 =
    rpc_validator_key_logic::api_error_code::MISSING_VALIDATOR_KEY;

/// Validator key extraction has failed.
pub const VALIDATOR_KEY_EXTRACTION: i32 =
    rpc_validator_key_logic::api_error_code::VALIDATOR_KEY_EXTRACTION;
