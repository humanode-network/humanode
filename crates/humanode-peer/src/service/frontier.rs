//! The Frontier related stuff.

use super::Block;
use sc_cli::SubstrateCli;
use sc_service::BasePath;

/// Create frontier dir.
pub fn database_dir(config: &sc_service::Configuration) -> std::path::PathBuf {
    let config_dir = config
        .base_path
        .as_ref()
        .map(|base_path| base_path.config_dir(config.chain_spec.id()))
        .unwrap_or_else(|| {
            BasePath::from_project("", "", &crate::cli::Root::executable_name())
                .config_dir(config.chain_spec.id())
        });
    config_dir.join("frontier").join("db")
}

/// Construct frontier backend.
pub fn open_backend(config: &sc_service::Configuration) -> Result<fc_db::Backend<Block>, String> {
    fc_db::Backend::<Block>::new(&fc_db::DatabaseSettings {
        source: fc_db::DatabaseSettingsSrc::RocksDb {
            path: database_dir(config),
            cache_size: 0,
        },
    })
}
