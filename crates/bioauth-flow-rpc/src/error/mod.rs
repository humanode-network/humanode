//! All bioauth flow error kinds that we expose in the RPC.

pub mod code;
pub mod data;
pub mod shared;
mod sign;

pub use self::sign::Error as Sign;
