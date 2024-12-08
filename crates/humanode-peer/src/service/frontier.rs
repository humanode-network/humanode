//! The Frontier related stuff.

use std::{path::Path, sync::Arc};

use fc_storage::OverrideHandle;
use humanode_runtime::opaque::Block;
use sc_client_api::backend::Backend;
use sc_service::Configuration;

use super::{FrontierBackend, FullClient, ServiceError};
use crate::configuration::{self, FrontierBackendType};

/// Create frontier dir.
pub fn db_config_dir(config: &sc_service::Configuration) -> std::path::PathBuf {
    config.base_path.config_dir(config.chain_spec.id())
}

/// Create frontier backend.
pub fn backend(
    config: &Configuration,
    client: Arc<FullClient>,
    fb_config: &configuration::FrontierBackend,
    eth_overrides: Arc<OverrideHandle<Block>>,
) -> Result<FrontierBackend, ServiceError> {
    match fb_config.frontier_backend_type {
        FrontierBackendType::KeyValue => Ok(FrontierBackend::KeyValue(fc_db::kv::Backend::open(
            Arc::clone(&client),
            &config.database,
            &db_config_dir(config),
        )?)),
        FrontierBackendType::Sql => {
            let db_path = db_config_dir(config).join("sql");
            std::fs::create_dir_all(&db_path)?;

            let backend = futures::executor::block_on(fc_db::sql::Backend::new(
                fc_db::sql::BackendConfig::Sqlite(fc_db::sql::SqliteBackendConfig {
                    path: Path::new("sqlite:///")
                        .join(db_path)
                        .join("frontier.db3")
                        .to_str()
                        .ok_or(ServiceError::Other("invalid sqlite path".to_owned()))?,
                    create_if_missing: true,
                    thread_count: fb_config.frontier_sql_backend_thread_count,
                    cache_size: fb_config.frontier_sql_backend_cache_size,
                }),
                fb_config.frontier_sql_backend_pool_size,
                std::num::NonZeroU32::new(fb_config.frontier_sql_backend_num_ops_timeout),
                Arc::clone(&eth_overrides),
            ))
            .map_err(|err| ServiceError::Application(err.into()))?;

            Ok(FrontierBackend::Sql(backend))
        }
    }
}

/// Default ethereum config.
pub struct DefaultEthConfig<C, BE>(std::marker::PhantomData<(C, BE)>);

impl<C, BE> fc_rpc::EthConfig<Block, C> for DefaultEthConfig<C, BE>
where
    C: sc_client_api::StorageProvider<Block, BE> + Sync + Send + 'static,
    BE: Backend<Block> + 'static,
{
    type EstimateGasAdapter = ();
    type RuntimeStorageOverride =
        fc_rpc::frontier_backend_client::SystemAccountId20StorageOverride<Block, C, BE>;
}
