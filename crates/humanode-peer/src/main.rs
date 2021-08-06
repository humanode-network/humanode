//! The Humanode Peer implementation, main executable entrypoint.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]

mod chain_spec;
mod cli;
mod command;
mod qrcode;
mod service;
mod validator_key;

fn main() -> sc_cli::Result<()> {
    command::run()
}
