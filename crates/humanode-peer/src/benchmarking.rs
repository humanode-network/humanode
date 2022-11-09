//! Setup code for [`crate::cli::run`] benchmarking commands which would otherwise bloat that module.
//!
//! Should only be used for benchmarking as it may break in other contexts.

use std::sync::Arc;
use std::time::Duration;

use frame_benchmarking_cli::ExtrinsicBuilder;
use frame_system_rpc_runtime_api::AccountNonceApi;
use humanode_runtime::{AccountId, Balance, BalancesCall, Runtime, SystemCall, opaque::Block};
use sc_client_api::BlockBackend;
use sp_api::ProvideRuntimeApi;
use sp_core::{Encode, Get, Pair, U256};
use sp_inherents::{InherentData, InherentDataProvider};
use sp_keyring::Sr25519Keyring;
use sp_runtime::{generic, OpaqueExtrinsic, SaturatedConversion, traits::Block as BlockT};


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
    timestamp
        .provide_inherent_data(&mut inherent_data)
        .map_err(|e| format!("creating timestamp inherent data: {:?}", e))?;

    let dynamic_fees =
        pallet_dynamic_fee::InherentDataProvider(U256::from(config.evm.target_gas_price));
    dynamic_fees
        .provide_inherent_data(&mut inherent_data)
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

    // Get the biggest period possible that satisfies 2^(k - 1) < BlockHashCount.
    let block_hash_count = <<Runtime as frame_system::Config>::BlockHashCount as Get<u32>>::get();
    let period = block_hash_count
        .checked_next_power_of_two()
        .map(|c| c / 2)
        .unwrap_or(2) as u64;

    let tip = 0;
    let extra: humanode_runtime::SignedExtra = (
        frame_system::CheckSpecVersion::<humanode_runtime::Runtime>::new(),
        frame_system::CheckTxVersion::<humanode_runtime::Runtime>::new(),
        frame_system::CheckGenesis::<humanode_runtime::Runtime>::new(),
        frame_system::CheckEra::<humanode_runtime::Runtime>::from(generic::Era::mortal(
            period,
            best_block.saturated_into(),
        )),
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
