//! Humanode peer configuration.

use std::borrow::Cow;

use crate::rpc_url::{RpcUrl, RpcUrlResolver};

/// Peer configuration, including both standard substrate configuration and our custom extensions.
pub struct Configuration {
    /// Standard substrate configuration.
    pub substrate: sc_service::Configuration,

    /// Bioauth flow configuration.
    /// If not defined, bioauth flows are unavailable.
    /// A lot of operations do not involve bioauth flows, so the configuration is not
    /// always required.
    pub bioauth_flow: Option<BioauthFlow>,

    /// EVM configuration.
    pub evm: Evm,

    /// Ethereum RPC configuration.
    pub ethereum_rpc: Option<EthereumRpc>,
}

/// Bioauth flow configuration parameters.
pub struct BioauthFlow {
    /// The RPC URL Resolver.
    /// Used for detecting the RPC URL in non-trivial cases.
    pub rpc_url_resolver: RpcUrlResolver,

    /// The URL to use for the web app.
    /// Used to print the QR Code to the console, so it can be optional, and if it's not defined
    /// we can assume user will take care of figuring out how to authenticate on its own.
    pub webapp_url: Option<String>,

    /// The URL to pass to the webapp to connect to the node RPC.
    /// If it's not defined we can assume user will take care of figuring out how
    /// to authenticate on its own.
    pub rpc_url: RpcUrl,

    /// The URL of robonode to authenticate with.
    pub robonode_url: String,
}

impl BioauthFlow {
    /// Obtain QR Code URL params.
    pub async fn qrcode_params(&self) -> Result<(&str, Cow<'_, str>), Cow<'static, str>> {
        let webapp_url = self.webapp_url.as_deref().ok_or("webapp URL is not set")?;
        let rpc_url = self.rpc_url_resolver.resolve(&self.rpc_url).await?;
        Ok((webapp_url, rpc_url))
    }
}

/// EVM configuration parameters.
pub struct Evm {
    /// The dynamic-fee pallet target gas price set by block author.
    pub target_gas_price: u64,
}

/// Ethereum RPC configuration parameters.
pub struct EthereumRpc {
    /// Maximum number of blocks to keep the log information available
    /// for querying via the RPC (from the latest block).
    pub max_past_logs: u32,

    /// Maximum number of stored filters.
    pub max_stored_filters: usize,

    /// Maximum fee history cache size.
    pub fee_history_limit: u64,

    /// A multiplier to allow larger gas limit in non-transactional execution.
    ///
    /// When using eth_call/eth_estimateGas, the maximum allowed gas limit will be
    /// block.gas_limit * execute_gas_limit_multiplier.
    pub execute_gas_limit_multiplier: u64,
}
