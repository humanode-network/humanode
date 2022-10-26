//! Setup code for [`crate::cli::run`] benchmarking commands which would otherwise bloat that module.
//!
//! Should only be used for benchmarking as it may break in other contexts.

use std::sync::Arc;
use std::time::Duration;

use frame_benchmarking_cli::ExtrinsicBuilder;
use humanode_runtime::{BalancesCall, SystemCall};
use sp_inherents::{InherentData, InherentDataProvider};
use sp_keyring::Sr25519Keyring;
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    MultiSignature, OpaqueExtrinsic,
};

use crate::service::create_extrinsic;
use crate::service::FullClient;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
type Signature = MultiSignature;
/// A way to identify an account on the chain. This is equivalent to public key of transaction signing scheme.
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
/// Balance of an account.
type Balance = u128;

/// Generates `System::Remark` extrinsics for the benchmarks.
///
/// Note: Should only be used for benchmarking.
pub struct RemarkBuilder {
    /// A shared full client instance.
    client: Arc<FullClient>,
}

impl RemarkBuilder {
    /// Creates a new [`Self`] from the given Client.
    pub fn new(client: Arc<FullClient>) -> Self {
        Self { client }
    }
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
    client: Arc<FullClient>,
    /// Destination account to receive.
    dest: AccountId,
    /// Value of the transfer.
    value: Balance,
}

impl TransferKeepAliveBuilder {
    /// Creates a new [`Self`] from the given Client.
    pub fn new(client: Arc<FullClient>, dest: AccountId, value: Balance) -> Self {
        Self {
            client,
            dest,
            value,
        }
    }
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
pub fn inherent_benchmark_data() -> sc_cli::Result<InherentData> {
    let mut inherent_data = InherentData::new();
    let d = Duration::from_millis(0);
    let timestamp = sp_timestamp::InherentDataProvider::new(d.into());

    timestamp
        .provide_inherent_data(&mut inherent_data)
        .map_err(|e| format!("creating inherent data: {:?}", e))?;
    Ok(inherent_data)
}
