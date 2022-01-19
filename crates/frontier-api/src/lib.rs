//! The runtime API for the frontier related stuff.

#![cfg_attr(not(feature = "std"), no_std)]
// `decl_runtime_apis` macro has issues.
#![allow(clippy::too_many_arguments, clippy::unnecessary_mut_passed)]

use codec::Decode;
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
    /// Runtime API for the transaction converter.
    pub trait TransactionConverterApi<Extrinsic: Decode> {
        /// Convert an ethereum transaction to an extrinsic.
        fn convert_transaction(transaction: pallet_ethereum::Transaction) -> Extrinsic;
    }
}
