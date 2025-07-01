//! Runtime builder script.
//!
//! The main goal of it is to compile the runtime for WASM, and prepare the files with
//! and embedded WASM blob for the native build.
//! It is transparent, is a sense that you won't notice if everything works out.

fn main() {
    #[cfg(feature = "std")]
    {
        let builder_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        substrate_wasm_builder::WasmBuilder::new()
            .with_project(format!("{builder_path}/../humanode-runtime/Cargo.toml"))
            .unwrap()
            .import_memory()
            .export_heap_base()
            .append_to_rust_flags("-C target-cpu=mvp")
            .append_to_rust_flags("-C target-feature=-sign-ext")
            .build()
    }
}
