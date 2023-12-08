//! The Frontier related stuff.

use std::{path::Path, sync::Arc};

use fc_storage::OverrideHandle;
use humanode_runtime::opaque::Block;
use sc_cli::SubstrateCli;
use sc_client_api::backend::Backend;
use sc_service::{BasePath, Configuration};

use super::{FrontierBackend, FullClient};
use crate::configuration::{EthereumRpc, FrontierBackendType};

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

pub fn frontier_backend(
    config: &Configuration,
    client: Arc<FullClient>,
    eth_rpc: &Option<EthereumRpc>,
    eth_overrides: Arc<OverrideHandle<Block>>,
) -> FrontierBackend {
    let key_value_frontier_backend = FrontierBackend::KeyValue(
        fc_db::kv::Backend::open(
            Arc::clone(&client),
            &config.database,
            &db_config_dir(config),
        )
        .unwrap(),
    );

    if let Some(eth_rpc) = eth_rpc {
        match eth_rpc.frontier_backend_type {
            FrontierBackendType::KeyValue => key_value_frontier_backend,
            FrontierBackendType::Sql => {
                let db_path = db_config_dir(config).join("sql");
                std::fs::create_dir_all(&db_path).expect("failed creating sql db directory");
                let backend = futures::executor::block_on(fc_db::sql::Backend::new(
                    fc_db::sql::BackendConfig::Sqlite(fc_db::sql::SqliteBackendConfig {
                        path: Path::new("sqlite:///")
                            .join(db_path)
                            .join("frontier.db3")
                            .to_str()
                            .unwrap(),
                        create_if_missing: true,
                        thread_count: eth_rpc.frontier_sql_backend_thread_count,
                        cache_size: eth_rpc.frontier_sql_backend_cache_size,
                    }),
                    eth_rpc.frontier_sql_backend_pool_size,
                    std::num::NonZeroU32::new(eth_rpc.frontier_sql_backend_num_ops_timeout),
                    Arc::clone(&eth_overrides),
                ))
                .unwrap_or_else(|err| panic!("failed creating sql backend: {:?}", err));
                FrontierBackend::Sql(backend)
            }
        }
    } else {
        key_value_frontier_backend
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
