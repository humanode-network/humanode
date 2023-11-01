//! The Frontier related stuff.

use humanode_runtime::opaque::Block;
use sc_cli::SubstrateCli;
use sc_client_api::backend::Backend;
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
