//! Storage related crypto helper functions.

use frame_support::{StorageHasher, Twox128};

/// Return encoded storage key based on provided module and method.
pub fn encoded_key(module: &str, method: &str) -> Vec<u8> {
    let mut encoded_storage_key = vec![];

    encoded_storage_key.extend(Twox128::hash(module.as_bytes()));
    encoded_storage_key.extend(Twox128::hash(method.as_bytes()));

    encoded_storage_key
}
