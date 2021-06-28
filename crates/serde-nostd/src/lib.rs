//! A utility crate that provides traits (usable in trait bounds) that resolve to serde in `std`,
//! and to nothing at `no_std`.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// When at std, Serialize and Deserialize.
#[cfg(feature = "std")]
pub trait SerDe: Serialize + for<'de> Deserialize<'de> {}

#[cfg(feature = "std")]
impl<T> SerDe for T where T: Serialize + for<'de> Deserialize<'de> {}

/// When at no_std, empty.
#[cfg(not(feature = "std"))]
pub trait SerDe {}

#[cfg(not(feature = "std"))]
impl<T> SerDe for T {}
