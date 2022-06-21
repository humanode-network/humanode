//! An adapter to allow passing additional params to the substrate commands and subcommands.

use super::params;
use crate::cli::{CliConfigurationExt, SubstrateCliConfigurationProvider};

/// The commands adapter.
#[derive(Debug, clap::Parser)]
pub struct Cmd<B, E>
where
    B: clap::Args + sc_cli::CliConfiguration,
    E: clap::Args,
{
    /// The base parameter.
    #[clap(flatten)]
    pub base: B,

    /// The extension.
    #[clap(flatten)]
    pub extra: E,
}

/// The subcommand adapter.
#[derive(Debug, clap::Parser)]
pub struct Subcommand<B, E>
where
    B: clap::Subcommand + sc_cli::CliConfiguration,
    E: clap::Args,
{
    /// The base parameter.
    #[clap(subcommand)]
    pub base: B,

    /// The extension.
    #[clap(flatten)]
    pub extra: E,
}

impl<B, E> SubstrateCliConfigurationProvider for Cmd<B, E>
where
    B: clap::Args + sc_cli::CliConfiguration,
    E: clap::Args,
{
    type SubstrateCliConfiguration = B;

    fn substrate_cli_configuration(&self) -> &Self::SubstrateCliConfiguration {
        &self.base
    }
}

impl<B, E> SubstrateCliConfigurationProvider for Subcommand<B, E>
where
    B: clap::Subcommand + sc_cli::CliConfiguration,
    E: clap::Args,
{
    type SubstrateCliConfiguration = B;

    fn substrate_cli_configuration(&self) -> &Self::SubstrateCliConfiguration {
        &self.base
    }
}

#[derive(Debug, clap::Parser)]
pub struct NoExtraParams;

impl<B> CliConfigurationExt for Cmd<B, NoExtraParams> where B: clap::Args + sc_cli::CliConfiguration {}
impl<B> CliConfigurationExt for Subcommand<B, NoExtraParams> where
    B: clap::Subcommand + sc_cli::CliConfiguration
{
}

#[derive(Debug, clap::Parser)]
pub struct ExtraEvmParams {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[clap(flatten)]
    pub evm_params: params::EvmParams,

    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[clap(flatten)]
    pub ethereum_rpc_params: params::EthereumRpcParams,
}

impl<B> CliConfigurationExt for Cmd<B, ExtraEvmParams>
where
    B: clap::Args + sc_cli::CliConfiguration,
{
    fn evm_params(&self) -> Option<&params::EvmParams> {
        Some(&self.extra.evm_params)
    }

    fn ethereum_rpc_params(&self) -> Option<&params::EthereumRpcParams> {
        Some(&self.extra.ethereum_rpc_params)
    }
}

impl<B> CliConfigurationExt for Subcommand<B, ExtraEvmParams>
where
    B: clap::Subcommand + sc_cli::CliConfiguration,
{
    fn evm_params(&self) -> Option<&params::EvmParams> {
        Some(&self.extra.evm_params)
    }

    fn ethereum_rpc_params(&self) -> Option<&params::EthereumRpcParams> {
        Some(&self.extra.ethereum_rpc_params)
    }
}
