//! The Humanode Peer implementation, main executable entrypoint.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]

mod dummy;
mod service;

fn main() {
    service::new_full(todo!()).unwrap();
}
