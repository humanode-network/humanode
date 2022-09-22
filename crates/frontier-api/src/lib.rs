//! The runtime API for the frontier related stuff.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Decode;

sp_api::decl_runtime_apis! {
    /// Runtime API for the transaction converter.
    pub trait TransactionConverterApi<Extrinsic: Decode> {
        /// Convert an ethereum transaction to an extrinsic.
        fn convert_transaction(transaction: pallet_ethereum::Transaction) -> Extrinsic;
    }
}
