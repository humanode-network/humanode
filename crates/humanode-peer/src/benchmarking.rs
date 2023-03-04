//! Setup code for [`crate::cli::run`] benchmarking commands which would otherwise bloat that module.
//!
//! Should only be used for benchmarking as it may break in other contexts.

use std::sync::Arc;
use std::time::Duration;

use frame_benchmarking_cli::ExtrinsicBuilder;
use frame_system_rpc_runtime_api::AccountNonceApi;
use humanode_runtime::{
    opaque::Block, utils::longest_era_for_block_hashes, AccountId, Balance, BalancesCall, Runtime,
    SystemCall, SLOT_DURATION,
};
use sc_client_api::BlockBackend;
use sp_api::ProvideRuntimeApi;
use sp_consensus_babe::SlotDuration;
use sp_core::{Encode, Pair, U256};
use sp_inherents::{InherentData, InherentDataProvider};
use sp_keyring::Sr25519Keyring;
use sp_runtime::{generic, traits::Block as BlockT, OpaqueExtrinsic, SaturatedConversion};

use crate::configuration::Configuration;
use crate::service::FullClient;

/// Generates `System::Remark` extrinsics for the benchmarks.
///
/// Note: Should only be used for benchmarking.
pub struct RemarkBuilder {
    /// A shared full client instance.
    pub client: Arc<FullClient>,
}

impl ExtrinsicBuilder for RemarkBuilder {
    fn pallet(&self) -> &str {
        "system"
    }

    fn extrinsic(&self) -> &str {
        "remark"
    }

    fn build(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
        let acc = Sr25519Keyring::Alice.pair();
        let extrinsic: OpaqueExtrinsic = create_extrinsic(
            self.client.as_ref(),
            acc,
            SystemCall::remark { remark: vec![] },
            Some(nonce),
        )
        .into();
        Ok(extrinsic)
    }
}

/// Generates `Balances::TransferKeepAlive` extrinsics for the benchmarks.
///
/// Note: Should only be used for benchmarking.
pub struct TransferKeepAliveBuilder {
    /// A shared full client instance.
    pub client: Arc<FullClient>,
    /// Destination account to receive.
    pub dest: AccountId,
    /// Value of the transfer.
    pub value: Balance,
}

impl ExtrinsicBuilder for TransferKeepAliveBuilder {
    fn pallet(&self) -> &str {
        "balances"
    }

    fn extrinsic(&self) -> &str {
        "transfer_keep_alive"
    }

    fn build(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
        let acc = Sr25519Keyring::Alice.pair();
        let extrinsic: OpaqueExtrinsic = create_extrinsic(
            self.client.as_ref(),
            acc,
            BalancesCall::transfer_keep_alive {
                dest: self.dest.clone().into(),
                value: self.value,
            },
            Some(nonce),
        )
        .into();
        Ok(extrinsic)
    }
}

/// Generates inherent data for the `benchmark overhead` command.
pub fn inherent_benchmark_data(config: &Configuration) -> sc_cli::Result<InherentData> {
    let mut inherent_data = InherentData::new();

    let d = Duration::from_millis(0);
    let timestamp = sp_timestamp::InherentDataProvider::new(d.into());
    futures::executor::block_on(timestamp.provide_inherent_data(&mut inherent_data))
        .map_err(|e| format!("creating timestamp inherent data: {:?}", e))?;

    let uncles =
        sp_authorship::InherentDataProvider::<<Block as BlockT>::Header>::check_inherents();
    futures::executor::block_on(uncles.provide_inherent_data(&mut inherent_data))
        .map_err(|e| format!("creating uncles inherent data: {:?}", e))?;

    let slot_duration = SlotDuration::from_millis(SLOT_DURATION);
    let slot = sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
        *timestamp,
        slot_duration,
    );
    futures::executor::block_on(slot.provide_inherent_data(&mut inherent_data))
        .map_err(|e| format!("creating slot inherent data: {:?}", e))?;

    let dynamic_fees =
        pallet_dynamic_fee::InherentDataProvider(U256::from(config.evm.target_gas_price));
    futures::executor::block_on(dynamic_fees.provide_inherent_data(&mut inherent_data))
        .map_err(|e| format!("creating dynamic fee inherent data: {:?}", e))?;
    Ok(inherent_data)
}

/// Create a transaction using the given call.
///
/// The transaction will be signed by the `sender`.
pub fn create_extrinsic(
    client: &FullClient,
    sender: sp_core::sr25519::Pair,
    function: impl Into<humanode_runtime::RuntimeCall>,
    maybe_nonce: Option<u32>,
) -> humanode_runtime::UncheckedExtrinsic {
    let function = function.into();
    let genesis_hash = client
        .block_hash(0)
        .ok()
        .flatten()
        .expect("Genesis block exists; qed");
    let best_hash = client.chain_info().best_hash;
    let best_block = client.chain_info().best_number;
    let nonce = maybe_nonce.unwrap_or_else(|| fetch_nonce(client, sender.clone()));

    let era = longest_era_for_block_hashes::<<Runtime as frame_system::Config>::BlockHashCount>(
        best_block.saturated_into(),
    );

    let tip = 0;
    let extra: humanode_runtime::SignedExtra = (
        frame_system::CheckSpecVersion::<humanode_runtime::Runtime>::new(),
        frame_system::CheckTxVersion::<humanode_runtime::Runtime>::new(),
        frame_system::CheckGenesis::<humanode_runtime::Runtime>::new(),
        frame_system::CheckEra::<humanode_runtime::Runtime>::from(era),
        frame_system::CheckNonce::<humanode_runtime::Runtime>::from(nonce),
        frame_system::CheckWeight::<humanode_runtime::Runtime>::new(),
        pallet_bioauth::CheckBioauthTx::<humanode_runtime::Runtime>::new(),
        pallet_transaction_payment::ChargeTransactionPayment::<humanode_runtime::Runtime>::from(
            tip,
        ),
        pallet_token_claims::CheckTokenClaim::<humanode_runtime::Runtime>::new(),
    );

    let raw_payload = humanode_runtime::SignedPayload::from_raw(
        function.clone(),
        extra.clone(),
        (
            humanode_runtime::VERSION.spec_version,
            humanode_runtime::VERSION.transaction_version,
            genesis_hash,
            best_hash,
            (),
            (),
            (),
            (),
            (),
        ),
    );

    let signature = raw_payload.using_encoded(|e| sender.sign(e));

    humanode_runtime::UncheckedExtrinsic::new_signed(
        function,
        sp_runtime::AccountId32::from(sender.public()).into(),
        humanode_runtime::Signature::Sr25519(signature),
        extra,
    )
}

/// Fetch the nonce of the given `account` from the chain state.
fn fetch_nonce(client: &FullClient, account: sp_core::sr25519::Pair) -> u32 {
    let best_hash = client.chain_info().best_hash;
    client
        .runtime_api()
        .account_nonce(&generic::BlockId::Hash(best_hash), account.public().into())
        .expect("Fetching account nonce failed")
}
