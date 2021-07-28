//! Core logic of the system.

use std::marker::PhantomData;

use facetec_api_client as ft;
use tokio::sync::Mutex;

use crate::sequence::Sequence;

mod common;
mod facetec_utils;
pub mod op_authenticate;
pub mod op_enroll;
pub mod op_get_facetec_device_sdk_params;
pub mod op_get_facetec_session_token;
#[cfg(test)]
mod tests;
mod traits;

pub use traits::*;

/// The overall generic logic.
pub struct Logic<S, PK> {
    /// The mutex over the locked portions of the logic.
    /// This way we're ensuring the operations can only be conducted under
    /// the lock.
    pub locked: Mutex<Locked<S, PK>>,
    /// The FaceTec Device SDK params to expose.
    pub facetec_device_sdk_params: FacetecDeviceSdkParams,
}

/// The inner state, to be hidden behind the mutex to ensure we don't have
/// access to it unless we lock the mutex.
pub struct Locked<S, PK> {
    /// The sequence number.
    pub sequence: Sequence,
    /// An execution ID, to be used together with sequence to guarantee unqiueness of the temporary
    /// enrollment external database IDs.
    pub execution_id: String,
    /// The client for the FaceTec Server API.
    pub facetec: ft::Client<crate::LoggingInspector>,
    /// The utility for signing the responses.
    pub signer: S,
    /// Public key type to use under the hood.
    pub public_key_type: PhantomData<PK>,
}

/// The FaceTec Device SDK params.
#[derive(Debug)]
pub struct FacetecDeviceSdkParams {
    /// The public FaceMap encription key.
    pub public_face_map_encryption_key: String,
    /// The device key identifier.
    pub device_key_identifier: String,
}
