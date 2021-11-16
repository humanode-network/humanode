//! Runtime builder script.
//!
//! The main goal of it is to compile the runtime for WASM, and prepare the files with
//! and embedded WASM blob for the native build.
//! It is transparent, is a sense that you won't notice if everything works out.

use substrate_wasm_builder::WasmBuilder;

fn main() {
    WasmBuilder::new()
        .with_current_project()
        .import_memory()
        .export_heap_base()
        .build()
}
