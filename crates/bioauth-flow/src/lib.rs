//! The bioauth flow implementation, aka the logic for communication between the humanode
//! (aka humanode-peer), the app on the handheld device that perform that biometric capture,
//! and the robonode server that's responsible for authenticating against the bioauth system.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]

pub mod flow;
