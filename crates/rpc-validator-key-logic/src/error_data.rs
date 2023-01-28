//! The validator related error kinds.

use jsonrpsee::core::Serialize;

/// Custom rpc error codes.
#[derive(Debug, Clone, Copy)]
pub struct ValidatorKeyNotAvailable;

impl Serialize for ValidatorKeyNotAvailable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: jsonrpsee::core::__reexports::serde::Serializer,
    {
        serde_json::json!({ "validatorKeyNotAvailable": true }).serialize(serializer)
    }
}
