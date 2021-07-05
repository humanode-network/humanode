//! Docker-powered Test Facetec Server instance.

use std::path::PathBuf;

use dockertest::Composition;

pub fn composition_from_env() -> Composition {
    let mut config = PathBuf::from(file!()).push("../../config/config.yaml");

    let composition = Composition::with_repository("facetec-server");
    composition.bind_mount(config, "/app/config.yaml");
    composition
}
