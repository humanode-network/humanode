//! The robonode requests related error.

use rpc_validator_key_logic::Error as ValidatorKeyError;

use super::sign::Error as SignError;

/// The robonode requests related error.
#[derive(Debug)]
pub enum Error<T: std::error::Error + 'static> {
    /// An error that can occur during validator key extraction.
    KeyExtraction(ValidatorKeyError),
    /// An error that can occur during signing process.
    Sign(SignError),
    /// An error that can occur during doing a call into robonode.
    RobonodeClient(robonode_client::Error<T>),
}
