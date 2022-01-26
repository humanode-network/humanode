//! Inherent data providers creator used at Aura import_queue and start_aura.

use sp_core::U256;
use std::time::Duration;

/// Create inherent data providers.
#[derive(Debug, Clone)]
pub struct Creator {
    /// Consensus slot duration.
    pub raw_slot_duration: Duration,
    /// Ethereum gas target price.
    pub eth_target_gas_price: u64,
}

#[async_trait::async_trait]
impl sp_inherents::CreateInherentDataProviders<super::Block, ()> for Creator {
    type InherentDataProviders = (
        sp_timestamp::InherentDataProvider,
        sp_consensus_aura::inherents::InherentDataProvider,
        pallet_dynamic_fee::InherentDataProvider,
    );

    async fn create_inherent_data_providers(
        &self,
        _parent: <super::Block as sp_runtime::traits::Block>::Hash,
        _extra_args: (),
    ) -> Result<Self::InherentDataProviders, Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

        let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
            *timestamp,
            self.raw_slot_duration,
        );

        let dynamic_fee =
            pallet_dynamic_fee::InherentDataProvider(U256::from(self.eth_target_gas_price));

        Ok((timestamp, slot, dynamic_fee))
    }
}
