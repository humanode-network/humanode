//! The runtime API for the EVM debug logic.

#![cfg_attr(not(feature = "std"), no_std)]
// TODO: fix clippy.
#![allow(missing_docs, clippy::too_many_arguments)]

use sp_core::{sp_std::vec::Vec, H160, H256, U256};

sp_api::decl_runtime_apis! {
    pub trait DebugRuntimeApi {
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
