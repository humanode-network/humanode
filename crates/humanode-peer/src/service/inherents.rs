//! Inherent data providers creator used at Babe import_queue and start_babe.

use sc_client_api::ProvideUncles;
use sc_service::Arc;
use sp_core::U256;
use sp_runtime::traits::Block;

/// Create inherent data providers.
#[derive(Debug)]
pub struct Creator<Client> {
    /// Consensus slot duration.
    pub raw_slot_duration: sp_consensus_babe::SlotDuration,
    /// Ethereum gas target price.
    pub eth_target_gas_price: u64,
    /// Client.
    pub client: Arc<Client>,
}

#[async_trait::async_trait]
impl<Client> sp_inherents::CreateInherentDataProviders<super::Block, ()> for Creator<Client>
where
    Client: Send + Sync,
    Client: ProvideUncles<super::Block>,
{
    type InherentDataProviders = (
        sp_timestamp::InherentDataProvider,
        sp_consensus_babe::inherents::InherentDataProvider,
        sp_authorship::InherentDataProvider<<super::Block as Block>::Header>,
        pallet_dynamic_fee::InherentDataProvider,
    );

    async fn create_inherent_data_providers(
        &self,
        parent: <super::Block as Block>::Hash,
        _extra_args: (),
    ) -> Result<Self::InherentDataProviders, Box<dyn std::error::Error + Send + Sync>> {
        let uncles =
            sc_consensus_uncles::create_uncles_inherent_data_provider(&*self.client, parent)?;

        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

        let slot =
            sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                *timestamp,
                self.raw_slot_duration,
            );

        let dynamic_fee =
            pallet_dynamic_fee::InherentDataProvider(U256::from(self.eth_target_gas_price));

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
