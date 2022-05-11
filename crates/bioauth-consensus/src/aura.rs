//! Aura consensus integration.

use std::{marker::PhantomData, sync::Arc};

use sp_api::{BlockId, ProvideRuntimeApi};
use sp_consensus_aura::{digests::CompatibleDigestItem, AuraApi};
use sp_runtime::traits::{Block as BlockT, Header};

/// Encapsulates block author extraction logic for aura consensus.
#[derive(Debug)]
pub struct BlockAuthorExtractor<Block: BlockT, Client, AuraAuthorityId> {
    /// Client provides access to the runtime.
    client: Arc<Client>,
    /// The type of the block used in the chain.
    _phantom_block: PhantomData<Block>,
    /// The type used an authority id in aura.
    _phantom_aura_authority_id: PhantomData<AuraAuthorityId>,
}

/// An error that can occur during block author extraction with the aura consensus.
#[derive(Debug, thiserror::Error)]
pub enum AuraBlockAuthorExtractorError {
    /// Unable to extract aura authorities from the chain state via the runtime.
    #[error("unable to extract aura authorities: {0}")]
    UnableToExtractAuthorities(sp_api::ApiError),
    /// Unable to obtain the slot from the block header.
    #[error("unable to obtaion the slot from the block header")]
    UnableToObtainSlot,
}

impl<Block: BlockT, Client, AuraAuthorityId> BlockAuthorExtractor<Block, Client, AuraAuthorityId> {
    /// Create a new [`AuraBlockAuthorExtractor`].
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            _phantom_block: PhantomData,
            _phantom_aura_authority_id: PhantomData,
        }
    }
}

impl<Block: BlockT, Client, AuraAuthorityId> Clone
    for BlockAuthorExtractor<Block, Client, AuraAuthorityId>
{
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            _phantom_block: PhantomData,
            _phantom_aura_authority_id: PhantomData,
        }
    }
}

impl<Block: BlockT, Client, AuraAuthorityId> crate::BlockAuthorExtractor
    for BlockAuthorExtractor<Block, Client, AuraAuthorityId>
where
    Client: ProvideRuntimeApi<Block>,
    Client::Api: AuraApi<Block, AuraAuthorityId>,
    AuraAuthorityId: codec::Codec + Clone,
{
    type Error = AuraBlockAuthorExtractorError;
    type Block = Block;
    type PublicKeyType = AuraAuthorityId;

    fn extract_block_author(
        &self,
        at: &BlockId<Self::Block>,
        block_header: &<Self::Block as BlockT>::Header,
    ) -> Result<AuraAuthorityId, Self::Error> {
        // Extract aura authorities list.
        let authorities = self
            .client
            .runtime_api()
            .authorities(at)
            .map_err(AuraBlockAuthorExtractorError::UnableToExtractAuthorities)?;

        // Extract the slot of a block.
        let slot = block_header
            .digest()
            .logs()
            .iter()
            .find_map(CompatibleDigestItem::<()>::as_aura_pre_digest)
            .ok_or(AuraBlockAuthorExtractorError::UnableToObtainSlot)?;

        // Author index in aura is current slot number mod authories.
        let author_index = *slot % authorities.len() as u64;

        // Determine the author of a block.
        let author_public_key = authorities
            .get(author_index as usize)
            .expect("author index is mod authories list len; qed");

        Ok(author_public_key.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::*;
    use mockall::*;
    use node_primitives::{Block, Header};
    use sp_api::{ApiError, ApiRef, NativeOrEncoded, ProvideRuntimeApi};
    use sp_runtime::{Digest, DigestItem};

    use super::*;

    type MockAuraAuthorityId = sp_consensus_aura::sr25519::AuthorityId;

    mock! {
        RuntimeApi {
            fn authorities(&self, _at: &sp_api::BlockId<Block>) -> Result<NativeOrEncoded<Vec<MockAuraAuthorityId>>, ApiError>;
            fn slot_duration(&self) -> sp_consensus_aura::SlotDuration;
        }
    }

    #[derive(Clone)]
    struct MockWrapperRuntimeApi(Arc<MockRuntimeApi>);

    sp_api::mock_impl_runtime_apis! {
        impl AuraApi<Block, MockAuraAuthorityId> for MockWrapperRuntimeApi {
            #[advanced]
            fn authorities(&self, at: &sp_api::BlockId<Block>) -> Result<
                NativeOrEncoded<Vec<MockAuraAuthorityId>>,
                ApiError
            > {
                self.0.authorities(at)
            }

            fn slot_duration() -> sp_consensus_aura::SlotDuration {
                self.0.slot_duration()
            }
        }
    }

    mock! {
        #[derive(Debug)]
        Client {}

        impl ProvideRuntimeApi<Block> for Client {
            type Api = MockWrapperRuntimeApi;

            fn runtime_api<'a>(&'a self) -> ApiRef<'a, MockWrapperRuntimeApi>;
        }
    }

    fn prepare_block_header_with_aura_pre_digest(empty_digest: bool) -> Header {
        let mut digest_items = vec![];
        if !empty_digest {
            let slot = sp_consensus_aura::Slot::from(1);
            let item = <DigestItem as CompatibleDigestItem<
                sp_consensus_aura::sr25519::AuthoritySignature,
            >>::aura_pre_digest(slot);
            digest_items.push(item);
        }

        Header {
            parent_hash: Default::default(),
            number: 1,
            state_root: Default::default(),
            extrinsics_root: Default::default(),
            digest: Digest { logs: digest_items },
        }
    }

    /// This test verifies aura block author extractor success when a respective runtime_api call (authorities)
    /// succeeds and the block header contains a correct aura digest.
    #[test]
    fn success() {
        let mut mock_client = MockClient::new();

        let mut mock_runtime_api = MockRuntimeApi::new();
        mock_runtime_api
            .expect_authorities()
            .returning(|_| Ok(NativeOrEncoded::from(vec![MockAuraAuthorityId::default()])));

        let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

        mock_client
            .expect_runtime_api()
            .returning(move || runtime_api.clone().into());

        let client = Arc::new(mock_client);

        let block_author_extractor: BlockAuthorExtractor<Block, _, MockAuraAuthorityId> =
            BlockAuthorExtractor::new(Arc::clone(&client));

        let res = crate::BlockAuthorExtractor::extract_block_author(
            &block_author_extractor,
            &sp_api::BlockId::Number(0),
            &prepare_block_header_with_aura_pre_digest(false),
        );

        // Drop the test object and all the mocks in it, effectively running the mock assertions.
        drop(block_author_extractor);
        // Unwrap the client from the Arc and drop it, ensuring it's mock assertions run too.
        drop(Arc::try_unwrap(client).unwrap());

        assert_eq!(res.unwrap(), MockAuraAuthorityId::default());
    }

    /// This test verifies aura block author extractor failure when a respective runtime_api call (authorities)
    /// succeeds, but the block header contains an incorrect digest.
    #[test]
    fn error_unable_to_obtain_slot() {
        let mut mock_client = MockClient::new();

        let mut mock_runtime_api = MockRuntimeApi::new();
        mock_runtime_api
            .expect_authorities()
            .returning(|_| Ok(NativeOrEncoded::from(vec![MockAuraAuthorityId::default()])));

        let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

        mock_client
            .expect_runtime_api()
            .returning(move || runtime_api.clone().into());

        let client = Arc::new(mock_client);

        let block_author_extractor: BlockAuthorExtractor<Block, _, MockAuraAuthorityId> =
            BlockAuthorExtractor::new(Arc::clone(&client));

        let res = crate::BlockAuthorExtractor::extract_block_author(
            &block_author_extractor,
            &sp_api::BlockId::Number(0),
            &prepare_block_header_with_aura_pre_digest(true),
        );

        // Drop the test object and all the mocks in it, effectively running the mock assertions.
        drop(block_author_extractor);
        // Unwrap the client from the Arc and drop it, ensuring it's mock assertions run too.
        drop(Arc::try_unwrap(client).unwrap());

        match res.unwrap_err() {
            AuraBlockAuthorExtractorError::UnableToObtainSlot => {}
            ref e => panic!(
                "assertion failed: `{:?}` does not match `{}`",
                e,
                stringify!(AuraBlockAuthorExtractorError::UnableToObtainSlot)
            ),
        }
    }

    /// This test verifies aura block author extractor failure when
    /// a respective runtime_api call (authorities) fails.
    #[test]
    fn runtime_error() {
        let mut mock_client = MockClient::new();

        let mut mock_runtime_api = MockRuntimeApi::new();
        mock_runtime_api.expect_authorities().returning(|_| {
            Err((Box::from("Test error") as Box<dyn std::error::Error + Send + Sync>).into())
        });

        let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

        mock_client
            .expect_runtime_api()
            .returning(move || runtime_api.clone().into());

        let client = Arc::new(mock_client);

        let block_author_extractor: BlockAuthorExtractor<Block, _, MockAuraAuthorityId> =
            BlockAuthorExtractor::new(Arc::clone(&client));

        let res = crate::BlockAuthorExtractor::extract_block_author(
            &block_author_extractor,
            &sp_api::BlockId::Number(0),
            &prepare_block_header_with_aura_pre_digest(false),
        );

        // Drop the test object and all the mocks in it, effectively running the mock assertions.
        drop(block_author_extractor);
        // Unwrap the client from the Arc and drop it, ensuring it's mock assertions run too.
        drop(Arc::try_unwrap(client).unwrap());

        match res.unwrap_err() {
            AuraBlockAuthorExtractorError::UnableToExtractAuthorities(e)
                if e.to_string() == "Test error" => {}
            ref e => panic!(
                "assertion failed: `{:?}` does not match `{}`",
                e,
                stringify!(AuraBlockAuthorExtractorError::UnableToExtractAuthorities(
                    "Test error"
                ))
            ),
        }
    }
}
