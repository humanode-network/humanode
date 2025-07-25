//! Stub binary target used to selectively build the WASM runtime via `build.rs`,
//! while ensuring consistent and correct feature resolution across the workspace.
//!
//! If you need to build only the `humanode-runtime` WASM, it will likely be incorrect to run
//! `cargo build --package humanode-runtime --lib`. While seemingly precise, `--package` would build
//! the WASM with the minimally acceptable — and likely untested — set of features. In contrast,
//! using `cargo build --bin humanode-runtime-wasm-builder` results in Cargo resolving features
//! at the workspace level. The runtime is compiled with the same feature set that has already been
//! tested through regular `--workspace` Cargo runs.

/// Stub binary entrypoint.
pub fn main() {}
