//! Utility functions for producing rpc error responses for the application level errors.

use jsonrpsee::{
    core::Error,
    types::{error::CallError, ErrorObject},
};
use serde::Serialize;

/// A simple error without the custom error data.
pub fn simple(code: i32, message: impl Into<String>) -> Error {
    raw(code, message, Option::<()>::None)
}

/// An error with the custom error data.
pub fn data<T: Serialize>(code: i32, message: impl Into<String>, data: T) -> Error {
    raw(code, message, Some(data))
}

/// A general form of an error with or without the custom error data.
pub fn raw<T: Serialize>(code: i32, message: impl Into<String>, data: Option<T>) -> Error {
    let error_object = ErrorObject::owned(code, message, data);
    Error::Call(CallError::Custom(error_object))
}
