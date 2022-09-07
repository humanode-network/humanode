//! The Frontier related stuff.

use sc_cli::SubstrateCli;
use sc_service::BasePath;

/// Create frontier dir.
pub fn db_config_dir(config: &sc_service::Configuration) -> std::path::PathBuf {
    config
        .base_path
        .as_ref()
        .map(|base_path| base_path.config_dir(config.chain_spec.id()))
        .unwrap_or_else(|| {
            BasePath::from_project("", "", &crate::cli::Root::executable_name())
                .config_dir(config.chain_spec.id())
        })
}
