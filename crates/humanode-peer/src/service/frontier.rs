//! The Frontier related stuff.

use sc_cli::SubstrateCli;
use sc_service::BasePath;

use super::Block;

/// Create frontier dir.
pub fn database_dir(config: &sc_service::Configuration, path: &str) -> std::path::PathBuf {
    let config_dir = config
        .base_path
        .as_ref()
        .map(|base_path| base_path.config_dir(config.chain_spec.id()))
        .unwrap_or_else(|| {
            BasePath::from_project("", "", &crate::cli::Root::executable_name())
                .config_dir(config.chain_spec.id())
        });
    config_dir.join("frontier").join(path)
}

/// Construct frontier backend.
pub fn open_backend(config: &sc_service::Configuration) -> Result<fc_db::Backend<Block>, String> {
    fc_db::Backend::<Block>::new(&fc_db::DatabaseSettings {
        source: match config.database {
            fc_db::DatabaseSource::RocksDb { .. } => fc_db::DatabaseSource::RocksDb {
                path: database_dir(config, "db"),
                cache_size: 0,
            },
            fc_db::DatabaseSource::ParityDb { .. } => fc_db::DatabaseSource::ParityDb {
                path: database_dir(config, "paritydb"),
            },
            fc_db::DatabaseSource::Auto { .. } => fc_db::DatabaseSource::Auto {
                rocksdb_path: database_dir(config, "db"),
                paritydb_path: database_dir(config, "paritydb"),
                cache_size: 0,
            },
            _ => return Err("Supported db sources: `rocksdb` | `paritydb` | `auto`".to_string()),
        },
    })
}
