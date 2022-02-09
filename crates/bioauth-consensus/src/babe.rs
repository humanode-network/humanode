//! Babe consensus integration.

use sp_api::{BlockId, ProvideRuntimeApi};
use sp_consensus_babe::{digests::CompatibleDigestItem, BabeApi};
use sp_runtime::traits::{Block as BlockT, Header};
use std::{marker::PhantomData, sync::Arc};

/// Encapsulates block author extraction logic for babe consensus.
#[derive(Debug)]
pub struct BlockAuthorExtractor<Block: BlockT, Client> {
    /// Client provides access to the runtime.
    client: Arc<Client>,
    /// The type of the block used in the chain.
    _phantom_block: PhantomData<Block>,
}

/// An error that can occur during block author extraction with the babe consensus.
#[derive(Debug, thiserror::Error)]
pub enum BabeBlockAuthorExtractorError {
    /// Unable to extract babe current epoch from the chain state via the runtime.
    #[error("unable to extract babe current epoch: {0}")]
    UnableToExtractCurrentEpoch(sp_api::ApiError),
    /// Unable to obtain the slot from the block header.
    #[error("unable to obtaion the slot from the block header")]
    UnableToObtainSlot,
    /// Unable to obtain the author.
    #[error("unable to obtaion the author from the slot")]
    UnableToObtainAuthor,
}

impl<Block: BlockT, Client> BlockAuthorExtractor<Block, Client> {
    /// Create a new [`BabeBlockAuthorExtractor`].
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            _phantom_block: PhantomData,
        }
    }
}

impl<Block: BlockT, Client> Clone for BlockAuthorExtractor<Block, Client> {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            _phantom_block: PhantomData,
        }
    }
}

impl<Block: BlockT, Client> crate::BlockAuthorExtractor for BlockAuthorExtractor<Block, Client>
where
    Client: ProvideRuntimeApi<Block>,
    Client::Api: BabeApi<Block>,
{
    type Error = BabeBlockAuthorExtractorError;
    type Block = Block;
    type PublicKeyType = sp_consensus_babe::AuthorityId;

    fn extract_block_author(
        &self,
        at: &BlockId<Self::Block>,
        block_header: &<Self::Block as BlockT>::Header,
    ) -> Result<Self::PublicKeyType, Self::Error> {
        // Extract babe current epoch.
        let current_epoch = self
            .client
            .runtime_api()
            .current_epoch(at)
            .map_err(BabeBlockAuthorExtractorError::UnableToExtractCurrentEpoch)?;

        // Get authorities list.
        let authorities = current_epoch.authorities;

        // Extract the slot of a block.
        let pre_digest = block_header
            .digest()
            .logs()
            .iter()
            .find_map(CompatibleDigestItem::as_babe_pre_digest)
            .ok_or(BabeBlockAuthorExtractorError::UnableToObtainSlot)?;

        // Determine the author of a block.
        let author = authorities
            .get(pre_digest.authority_index() as usize)
            .ok_or(BabeBlockAuthorExtractorError::UnableToObtainAuthor)?;

        let author_public_key = author.0.clone();

        Ok(author_public_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;
    use node_primitives::{Block, Header};
    use sp_api::{ApiError, ApiRef, NativeOrEncoded, ProvideRuntimeApi};
    use sp_runtime::{Digest, DigestItem};
    use std::sync::Arc;

    mock! {
        RuntimeApi {
            fn configuration(&self) -> sp_consensus_babe::BabeGenesisConfiguration;
            fn current_epoch_start(&self) -> sp_consensus_babe::Slot;
            fn current_epoch(&self, _at: &sp_api::BlockId<Block>) -> Result<NativeOrEncoded<sp_consensus_babe::Epoch>, ApiError>;
            fn next_epoch(&self) -> sp_consensus_babe::Epoch;
            fn generate_key_ownership_proof(
                &self,
                _slot: sp_consensus_babe::Slot,
                _authority_id: sp_consensus_babe::AuthorityId,
            ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof>;
            fn submit_report_equivocation_unsigned_extrinsic(
                &self,
                _equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
                _key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
            ) -> Option<()>;

        }
    }

    #[derive(Clone)]
    struct MockWrapperRuntimeApi(Arc<MockRuntimeApi>);

    sp_api::mock_impl_runtime_apis! {
        impl BabeApi<Block> for MockWrapperRuntimeApi {
            fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
                self.0.configuration()
            }

            fn current_epoch_start() -> sp_consensus_babe::Slot {
                self.0.current_epoch_start()
            }

            #[advanced]
            fn current_epoch(&self, at: &sp_api::BlockId<Block>) -> Result<
                NativeOrEncoded<sp_consensus_babe::Epoch>,
                ApiError
            > {
                self.0.current_epoch(at)
            }

            fn next_epoch() -> sp_consensus_babe::Epoch {
                self.0.next_epoch()
            }

            fn generate_key_ownership_proof(
                &self,
                slot: sp_consensus_babe::Slot,
                authority_id: sp_consensus_babe::AuthorityId,
            ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
                self.0.generate_key_ownership_proof(slot, authority_id)
            }

            fn submit_report_equivocation_unsigned_extrinsic(
                &self,
                equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
                key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
            ) -> Option<()> {
                self.0.submit_report_equivocation_unsigned_extrinsic(equivocation_proof, key_owner_proof)
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

    fn prepare_block_header_with_babe_pre_digest(
        empty_digest: bool,
        authority_index: u32,
    ) -> Header {
        let mut digest_items = vec![];
        if !empty_digest {
            let slot = sp_consensus_babe::Slot::from(1);
            let primary_pre_digest = sp_consensus_babe::digests::SecondaryPlainPreDigest {
                authority_index,
                slot,
            };
            let pre_digest =
                sp_consensus_babe::digests::PreDigest::SecondaryPlain(primary_pre_digest);
            let item = <DigestItem as CompatibleDigestItem>::babe_pre_digest(pre_digest);
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

    fn prepare_epoch() -> sp_consensus_babe::Epoch {
        sp_consensus_babe::Epoch {
            epoch_index: Default::default(),
            start_slot: Default::default(),
            duration: Default::default(),
            authorities: vec![(
                sp_consensus_babe::AuthorityId::default(),
                Default::default(),
            )],
            randomness: Default::default(),
            config: sp_consensus_babe::BabeEpochConfiguration {
                c: Default::default(),
                allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
            },
        }
    }

    /// This test verifies babe block author extractor success when a respective runtime_api call (current_epoch)
    /// succeeds and the block header contains a correct babe digest.
    #[test]
    fn success() {
        let mut mock_client = MockClient::new();

        let mut mock_runtime_api = MockRuntimeApi::new();
        mock_runtime_api
            .expect_current_epoch()
            .returning(|_| Ok(NativeOrEncoded::from(prepare_epoch())));

        let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

        mock_client
            .expect_runtime_api()
            .returning(move || runtime_api.clone().into());

        let client = Arc::new(mock_client);

        let block_author_extractor: BlockAuthorExtractor<Block, _> =
            BlockAuthorExtractor::new(Arc::clone(&client));

        let res = crate::BlockAuthorExtractor::extract_block_author(
            &block_author_extractor,
            &sp_api::BlockId::Number(0),
            &prepare_block_header_with_babe_pre_digest(false, 0),
        );

        // Drop the test object and all the mocks in it, effectively running the mock assertions.
        drop(block_author_extractor);
        // Unwrap the client from the Arc and drop it, ensuring it's mock assertions run too.
        drop(Arc::try_unwrap(client).unwrap());

        assert_eq!(res.unwrap(), sp_consensus_babe::AuthorityId::default());
    }

    /// This test verifies babe block author extractor failure when
    /// a respective runtime_api call (current_epoch) fails.
    #[test]
    fn runtime_error() {
        let mut mock_client = MockClient::new();

        let mut mock_runtime_api = MockRuntimeApi::new();
        mock_runtime_api.expect_current_epoch().returning(|_| {
            Err((Box::from("Test error") as Box<dyn std::error::Error + Send + Sync>).into())
        });

        let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

        mock_client
            .expect_runtime_api()
            .returning(move || runtime_api.clone().into());

        let client = Arc::new(mock_client);

        let block_author_extractor: BlockAuthorExtractor<Block, _> =
            BlockAuthorExtractor::new(Arc::clone(&client));

        let res = crate::BlockAuthorExtractor::extract_block_author(
            &block_author_extractor,
            &sp_api::BlockId::Number(0),
            &prepare_block_header_with_babe_pre_digest(false, 0),
        );

        // Drop the test object and all the mocks in it, effectively running the mock assertions.
        drop(block_author_extractor);
        // Unwrap the client from the Arc and drop it, ensuring it's mock assertions run too.
        drop(Arc::try_unwrap(client).unwrap());

        match res.unwrap_err() {
            BabeBlockAuthorExtractorError::UnableToExtractCurrentEpoch(e)
                if e.to_string() == "Test error" => {}
            ref e => panic!(
                "assertion failed: `{:?}` does not match `{}`",
                e,
                stringify!(BabeBlockAuthorExtractorError::UnableToExtractCurrentEpoch(
                    "Test error"
                ))
            ),
        }
    }

    /// This test verifies babe block author extractor failure when a respective runtime_api call (current_epoch)
    /// succeeds, but the block header contains an incorrect digest.
    #[test]
    fn error_unable_to_obtain_slot() {
        let mut mock_client = MockClient::new();

        let mut mock_runtime_api = MockRuntimeApi::new();
        mock_runtime_api
            .expect_current_epoch()
            .returning(|_| Ok(NativeOrEncoded::from(prepare_epoch())));

        let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

        mock_client
            .expect_runtime_api()
            .returning(move || runtime_api.clone().into());

        let client = Arc::new(mock_client);

        let block_author_extractor: BlockAuthorExtractor<Block, _> =
            BlockAuthorExtractor::new(Arc::clone(&client));

        let res = crate::BlockAuthorExtractor::extract_block_author(
            &block_author_extractor,
            &sp_api::BlockId::Number(0),
            &prepare_block_header_with_babe_pre_digest(true, 0),
        );

        // Drop the test object and all the mocks in it, effectively running the mock assertions.
        drop(block_author_extractor);
        // Unwrap the client from the Arc and drop it, ensuring it's mock assertions run too.
        drop(Arc::try_unwrap(client).unwrap());

        match res.unwrap_err() {
            BabeBlockAuthorExtractorError::UnableToObtainSlot => {}
            ref e => panic!(
                "assertion failed: `{:?}` does not match `{}`",
                e,
                stringify!(BabeBlockAuthorExtractorError::UnableToObtainSlot)
            ),
        }
    }

    /// This test verifies babe block author extractor failure when a respective runtime_api call (current_epoch)
    /// succeeds, slot has been extracted, but the author can't be extracted.
    #[test]
    fn error_unable_to_obtain_author() {
        let mut mock_client = MockClient::new();

        let mut mock_runtime_api = MockRuntimeApi::new();
        mock_runtime_api
            .expect_current_epoch()
            .returning(|_| Ok(NativeOrEncoded::from(prepare_epoch())));

        let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

        mock_client
            .expect_runtime_api()
            .returning(move || runtime_api.clone().into());

        let client = Arc::new(mock_client);

        let block_author_extractor: BlockAuthorExtractor<Block, _> =
            BlockAuthorExtractor::new(Arc::clone(&client));

        let res = crate::BlockAuthorExtractor::extract_block_author(
            &block_author_extractor,
            &sp_api::BlockId::Number(0),
            &prepare_block_header_with_babe_pre_digest(false, 1),
        );

        // Drop the test object and all the mocks in it, effectively running the mock assertions.
        drop(block_author_extractor);
        // Unwrap the client from the Arc and drop it, ensuring it's mock assertions run too.
        drop(Arc::try_unwrap(client).unwrap());

        match res.unwrap_err() {
            BabeBlockAuthorExtractorError::UnableToObtainAuthor => {}
            ref e => panic!(
                "assertion failed: `{:?}` does not match `{}`",
                e,
                stringify!(BabeBlockAuthorExtractorError::UnableToObtainAuthor)
            ),
        }
    }
}
