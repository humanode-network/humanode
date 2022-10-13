//! Inherent data providers creator used at Babe import_queue and start_babe.

use sc_client_api::ProvideUncles;
use sc_service::Arc;
use sp_core::U256;
use sp_runtime::traits::Block;

/// Create inherent data providers for block creation.
#[derive(Debug)]
pub struct Creator<Client> {
    /// Consensus slot duration.
    pub raw_slot_duration: sp_consensus_babe::SlotDuration,
    /// Ethereum gas target price.
    pub eth_target_gas_price: u64,
    /// Client.
    pub client: Arc<Client>,
}

/// Inherent data providers, common for both [`sp_inherents::CreateInherentDataProviders`]
/// implementations.
type InherentDataProviders = (
    sp_timestamp::InherentDataProvider,
    sp_consensus_babe::inherents::InherentDataProvider,
    sp_authorship::InherentDataProvider<<super::Block as Block>::Header>,
    pallet_dynamic_fee::InherentDataProvider,
);

/// The inherents creator for block production.
#[derive(Debug, Clone)]
pub struct ForProduction<T>(pub T);

/// The inherents creator for block import.
pub struct ForImport<T>(pub T);

#[async_trait::async_trait]
impl<Client> sp_inherents::CreateInherentDataProviders<super::Block, ()>
    for ForProduction<Creator<Client>>
where
    Client: Send + Sync,
    Client: ProvideUncles<super::Block>,
{
    type InherentDataProviders = InherentDataProviders;

    async fn create_inherent_data_providers(
        &self,
        parent: <super::Block as Block>::Hash,
        _extra_args: (),
    ) -> Result<Self::InherentDataProviders, Box<dyn std::error::Error + Send + Sync>> {
        let uncles =
            sc_consensus_uncles::create_uncles_inherent_data_provider(&*self.0.client, parent)?;

        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

        let revive_timestamp: u64 = std::env::var("REVIVE_TIMESTAMP").expect("REVIVE_TIMESTAMP not set").parse().expect("REVIVE_TIMESTAMP should be u64");
 		let fork_timestamp: u64 = std::env::var("FORK_TIMESTAMP").expect("FORK_TIMESTAMP not set").parse().expect("FORK_TIMESTAMP should be u64");
 		const WARP_FACTOR: u64 = 12;

 		let time_since_revival = timestamp.timestamp().saturating_sub(revive_timestamp);
 		let warped_timestamp = fork_timestamp + WARP_FACTOR * time_since_revival;

         tracing::debug!(target: "babe", message = format!("timestamp warped: {:?} to {:?} ({:?} since revival)",
         timestamp.timestamp(),
         warped_timestamp,
         time_since_revival)
     );

        let timestamp = timestamp.timestamp().min(warped_timestamp.into());

        let timestamp = sp_timestamp::InherentDataProvider::new(timestamp);

        let slot =
            sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                *timestamp,
                self.0.raw_slot_duration,
            );

        let dynamic_fee =
            pallet_dynamic_fee::InherentDataProvider(U256::from(self.0.eth_target_gas_price));

        Ok((timestamp, slot, uncles, dynamic_fee))
    }
}

#[async_trait::async_trait]
impl<Client> sp_inherents::CreateInherentDataProviders<super::Block, ()>
    for ForImport<Creator<Client>>
where
    Client: Send + Sync,
    Client: ProvideUncles<super::Block>,
{
    type InherentDataProviders = InherentDataProviders;

    async fn create_inherent_data_providers(
        &self,
        _parent: <super::Block as Block>::Hash,
        _extra_args: (),
    ) -> Result<Self::InherentDataProviders, Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

        let revive_timestamp: u64 = std::env::var("REVIVE_TIMESTAMP").expect("REVIVE_TIMESTAMP not set").parse().expect("REVIVE_TIMESTAMP should be u64");
 		let fork_timestamp: u64 = std::env::var("FORK_TIMESTAMP").expect("FORK_TIMESTAMP not set").parse().expect("FORK_TIMESTAMP should be u64");
 		const WARP_FACTOR: u64 = 12;

 		let time_since_revival = timestamp.timestamp().saturating_sub(revive_timestamp);
 		let warped_timestamp = fork_timestamp + WARP_FACTOR * time_since_revival;

         tracing::debug!(target: "babe", message = format!("timestamp warped: {:?} to {:?} ({:?} since revival)",
         timestamp.timestamp(),
         warped_timestamp,
         time_since_revival)
     );

        let timestamp = timestamp.timestamp().min(warped_timestamp.into());

        let timestamp = sp_timestamp::InherentDataProvider::new(timestamp);

        let slot =
            sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                *timestamp,
                self.0.raw_slot_duration,
            );

        let uncles =
            sp_authorship::InherentDataProvider::<<super::Block as Block>::Header>::check_inherents(
            );

        let dynamic_fee =
            pallet_dynamic_fee::InherentDataProvider(U256::from(self.0.eth_target_gas_price));

        Ok((timestamp, slot, uncles, dynamic_fee))
    }
}

impl<Client> Clone for Creator<Client> {
    fn clone(&self) -> Self {
        Self {
            raw_slot_duration: self.raw_slot_duration,
            eth_target_gas_price: self.eth_target_gas_price,
            client: Arc::clone(&self.client),
        }
    }
}
