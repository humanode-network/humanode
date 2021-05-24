// This file is part of Substrate.

// Copyright (C) 2018-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Module implementing the logic for verifying and importing Dummy blocks.

use std::{
	sync::Arc, marker::PhantomData, hash::Hash, fmt::Debug, collections::HashMap,
};
use substrate_prometheus_endpoint::Registry;
use codec::{Encode, Decode, Codec};
use sp_consensus::{
	BlockImport, BlockImportParams,
	BlockOrigin, Error as ConsensusError, BlockCheckParams, ImportResult,
	import_queue::{
		Verifier, BasicQueue, DefaultImportQueue, BoxJustificationImport,
	},
};
use sc_client_api::{BlockOf, UsageProvider, backend::AuxStore};
use sp_blockchain::{well_known_cache_keys::{self, Id as CacheKeyId}, ProvideCache, HeaderBackend};
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_runtime::{generic::{BlockId, OpaqueDigestItemId}, Justifications};
use sp_runtime::traits::{Block as BlockT, Header, DigestItemFor};
use sp_api::ProvideRuntimeApi;
use sp_core::crypto::Pair;
use sp_inherents::{CreateInherentDataProviders, InherentDataProvider as _};
use sp_api::ApiExt;


/// A verifier for Dummy blocks.
pub struct DummyVerifier<C, P, CIDP> {
	client: Arc<C>,
	phantom: PhantomData<P>,
	create_inherent_data_providers: CIDP,
}

impl<C, P, CIDP> DummyVerifier<C, P, CIDP> {
	pub(crate) fn new(
		client: Arc<C>,
		create_inherent_data_providers: CIDP,
	) -> Self {
		Self {
			client,
			create_inherent_data_providers,
			phantom: PhantomData,
		}
	}
}


#[async_trait::async_trait]
impl<B: BlockT, C, P, CIDP> Verifier<B> for DummyVerifier<C, P, CIDP> where
	C: ProvideRuntimeApi<B> +
		Send +
		Sync +
		sc_client_api::backend::AuxStore +
		ProvideCache<B> +
		BlockOf,
	P: Pair + Send + Sync + 'static,
	P::Public: Send + Sync + Hash + Eq + Clone + Decode + Encode + Debug + 'static,
	P::Signature: Encode + Decode,
	CIDP: CreateInherentDataProviders<B, ()> + Send + Sync,
{
	async fn verify(
		&mut self,
		origin: BlockOrigin,
		header: B::Header,
		justifications: Option<Justifications>,
		mut body: Option<Vec<B::Extrinsic>>,
	) -> Result<(BlockImportParams<B, ()>, Option<Vec<(CacheKeyId, Vec<u8>)>>), String> {
	
		// TODO: implement verify logic

		let import_block = BlockImportParams::new(origin, header);

		Ok((import_block, None))
		
	}
}


/// Parameters of [`import_queue`].
pub struct ImportQueueParams<'a, Block, I, C, S, CIDP> {
	/// The block import to use.
	pub block_import: I,
	/// The justification import.
	pub justification_import: Option<BoxJustificationImport<Block>>,
	/// The client to interact with the chain.
	pub client: Arc<C>,
	/// Something that can create the inherent data providers.
	pub create_inherent_data_providers: CIDP,
	/// The spawner to spawn background tasks.
	pub spawner: &'a S,
	/// The prometheus registry.
	pub registry: Option<&'a Registry>,
}

/// Start an import queue for the Dummy consensus algorithm.
pub fn import_queue<'a, P, Block, I, C, S, CIDP>(
	ImportQueueParams {
		block_import,
		justification_import,
		client,
		create_inherent_data_providers,
		spawner,
		registry,
	}: ImportQueueParams<'a, Block, I, C, S, CIDP>
) -> Result<DefaultImportQueue<Block, C>, sp_consensus::Error> where
	Block: BlockT,
	C: 'static
		+ ProvideRuntimeApi<Block>
		+ BlockOf
		+ ProvideCache<Block>
		+ Send
		+ Sync
		+ AuxStore
		+ UsageProvider<Block>
		+ HeaderBackend<Block>,
	I: BlockImport<Block, Error=ConsensusError, Transaction = sp_api::TransactionFor<C, Block>>
		+ Send
		+ Sync
		+ 'static,
	P: Pair + Send + Sync + 'static,
	P::Public: Clone + Eq + Send + Sync + Hash + Debug + Encode + Decode,
	P::Signature: Encode + Decode,
	S: sp_core::traits::SpawnEssentialNamed,
	CIDP: CreateInherentDataProviders<Block, ()> + Sync + Send + 'static,
{

	let verifier = build_verifier::<P, _, _>(
		BuildVerifierParams {
			client,
			create_inherent_data_providers,
		},
	);

	Ok(BasicQueue::new(
		verifier,
		Box::new(block_import),
		justification_import,
		spawner,
		registry,
	))
}

/// Parameters of [`build_verifier`].
pub struct BuildVerifierParams<C, CIDP> {
	/// The client to interact with the chain.
	pub client: Arc<C>,
	/// Something that can create the inherent data providers.
	pub create_inherent_data_providers: CIDP,
}

/// Build the [`DummyVerifier`]
pub fn build_verifier<P, C, CIDP>(
	BuildVerifierParams {
		client,
		create_inherent_data_providers,
	}: BuildVerifierParams<C, CIDP>
) -> DummyVerifier<C, P, CIDP> {
	DummyVerifier::<_, P, _>::new(
		client,
		create_inherent_data_providers,
	)
}


/// A block-import handler for Dummy.
pub struct DummyBlockImport<Block: BlockT, C, I: BlockImport<Block>, P> {
	inner: I,
	client: Arc<C>,
	_phantom: PhantomData<(Block, P)>,
}

impl<Block: BlockT, C, I: Clone + BlockImport<Block>, P> Clone for DummyBlockImport<Block, C, I, P> {
	fn clone(&self) -> Self {
		DummyBlockImport {
			inner: self.inner.clone(),
			client: self.client.clone(),
			_phantom: PhantomData,
		}
	}
}

impl<Block: BlockT, C, I: BlockImport<Block>, P> DummyBlockImport<Block, C, I, P> {
	/// New dummy block import.
	pub fn new(
		inner: I,
		client: Arc<C>,
	) -> Self {
		Self {
			inner,
			client,
			_phantom: PhantomData,
		}
	}
}

#[async_trait::async_trait]
impl<Block: BlockT, C, I, P> BlockImport<Block> for DummyBlockImport<Block, C, I, P> where
	I: BlockImport<Block, Transaction = sp_api::TransactionFor<C, Block>> + Send + Sync,
	I::Error: Into<ConsensusError>,
	C: HeaderBackend<Block> + ProvideRuntimeApi<Block>,
	P: Pair + Send + Sync + 'static,
	P::Public: Clone + Eq + Send + Sync + Hash + Debug + Encode + Decode,
	P::Signature: Encode + Decode,
	sp_api::TransactionFor<C, Block>: Send + 'static,
{
	type Error = ConsensusError;
	type Transaction = sp_api::TransactionFor<C, Block>;

	async fn check_block(
		&mut self,
		block: BlockCheckParams<Block>,
	) -> Result<ImportResult, Self::Error> {
		self.inner.check_block(block).await.map_err(Into::into)
	}

	async fn import_block(
		&mut self,
		block: BlockImportParams<Block, Self::Transaction>,
		new_cache: HashMap<CacheKeyId, Vec<u8>>,
	) -> Result<ImportResult, Self::Error> {
		
		// TODO: implement required logic

		self.inner.import_block(block, new_cache).await.map_err(Into::into)
	}
}