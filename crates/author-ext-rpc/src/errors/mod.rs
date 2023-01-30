//! All author extension related error kinds that we expose in the RPC.

pub mod get_validator_public_key;
pub mod set_keys;

/// Custom rpc error codes.
pub mod api_error_code {
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
}
