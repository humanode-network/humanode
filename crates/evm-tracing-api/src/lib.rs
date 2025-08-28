//! The runtime API for the EVM tracing logic.

#![cfg_attr(not(feature = "std"), no_std)]

use ethereum::TransactionV2 as Transaction;
use sp_core::{sp_std::vec::Vec, H160, H256, U256};

sp_api::decl_runtime_apis! {
    /// Runtime API for the EVM tracing logic.
    pub trait EvmTracingApi {
        /// Trace transaction.
        fn trace_transaction(
            extrinsics: Vec<Block::Extrinsic>,
            transaction: &Transaction,
            header: &Block::Header,
        ) -> Result<(), sp_runtime::DispatchError>;

        /// Trace block.
        fn trace_block(
            extrinsics: Vec<Block::Extrinsic>,
            known_transactions: Vec<H256>,
            header: &Block::Header,
        ) -> Result<(), sp_runtime::DispatchError>;

        /// Trace call execution.
        // Allow too many arguments to pass them in the way used at EVM runner call.
        #[allow(clippy::too_many_arguments)]
        fn trace_call(
            header: &Block::Header,
            from: H160,
            to: H160,
            data: Vec<u8>,
            value: U256,
            gas_limit: U256,
            max_fee_per_gas: Option<U256>,
            max_priority_fee_per_gas: Option<U256>,
            nonce: Option<U256>,
            access_list: Option<Vec<(H160, Vec<H256>)>>,
        ) -> Result<(), sp_runtime::DispatchError>;
    }
}
