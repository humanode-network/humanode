//! Inherent data providers creator used at Babe `import_queue` and `start_babe`.

use sc_client_api::ProvideUncles;
use sc_service::Arc;
use sp_runtime::traits::Block;

use crate::time_warp::TimeWarp;

/// Create inherent data providers for block creation.
#[derive(Debug)]
pub struct Creator<Client> {
    /// Consensus slot duration.
    pub raw_slot_duration: sp_consensus_babe::SlotDuration,
    /// Time warp peer mode.
    pub time_warp: Option<TimeWarp>,
    /// Client.
    pub client: Arc<Client>,
}

/// Inherent data providers, common for both [`sp_inherents::CreateInherentDataProviders`]
/// implementations.
type InherentDataProviders = (
    sp_consensus_babe::inherents::InherentDataProvider,
    sp_timestamp::InherentDataProvider,
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
        _parent: <super::Block as Block>::Hash,
        _extra_args: (),
    ) -> Result<Self::InherentDataProviders, Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

        let timestamp = if let Some(time_warp) = &self.0.time_warp {
            time_warp.apply_time_warp(timestamp.timestamp())
        } else {
            timestamp
        };

        let slot =
            sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                *timestamp,
                self.0.raw_slot_duration,
            );

        Ok((slot, timestamp))
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

        let timestamp = if let Some(time_warp) = &self.0.time_warp {
            time_warp.apply_time_warp(timestamp.timestamp())
        } else {
            timestamp
        };

        let slot =
            sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                *timestamp,
                self.0.raw_slot_duration,
            );

        Ok((slot, timestamp))
    }
}

impl<Client> Clone for Creator<Client> {
    fn clone(&self) -> Self {
        Self {
            raw_slot_duration: self.raw_slot_duration,
            time_warp: self.time_warp.clone(),
            client: Arc::clone(&self.client),
        }
    }
}

/// Sealing case related inherent data providers.
pub mod sealing {
    use std::cell::RefCell;

    thread_local!(static TIMESTAMP: RefCell<u64> = RefCell::new(0));

    /// Provide a simulated duration starting at 0 in millisecond for timestamp inherent.
    /// Each call will increment timestamp by slot duration making Babe think time has passed.
    pub struct SimulatedTimestampInherentDataProvider;

    #[async_trait::async_trait]
    impl sp_inherents::InherentDataProvider for SimulatedTimestampInherentDataProvider {
        async fn provide_inherent_data(
            &self,
            inherent_data: &mut sp_inherents::InherentData,
        ) -> Result<(), sp_inherents::Error> {
            TIMESTAMP.with(|x| {
                let current_timestamp = *x.borrow();
                *x.borrow_mut() = current_timestamp
                    .checked_add(humanode_runtime::SLOT_DURATION)
                    .expect("operation is under control, sealing case is allowed only in dev mode");
                inherent_data.put_data(sp_timestamp::INHERENT_IDENTIFIER, &*x.borrow())
            })
        }

        async fn try_handle_error(
            &self,
            _identifier: &sp_inherents::InherentIdentifier,
            _error: &[u8],
        ) -> Option<Result<(), sp_inherents::Error>> {
            // Never report errors in sealing dev mode.
            None
        }
    }
}
