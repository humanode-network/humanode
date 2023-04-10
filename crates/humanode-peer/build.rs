//! Node build script.

fn main() {
    vergen::EmitBuilder::builder()
        .fail_on_error()
        .git_sha(false)
        .cargo_features()
        .cargo_debug()
        .emit()
        .unwrap();
}
