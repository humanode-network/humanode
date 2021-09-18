//! The runtime API for the bioauth consensus.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]
#![cfg_attr(not(feature = "std"), no_std)]

/// APIs module, to work around clippy issues.
mod apis {
    #![allow(clippy::too_many_arguments, clippy::unnecessary_mut_passed)]

    sp_api::decl_runtime_apis! {
        /// Runtime API for the bioauth consensus.
        pub trait BioauthConsensusApi<Id: codec::Decode> {
            /// Get biometric-authenticated ids for the current block.
            fn ids() -> Vec<Id>;
        }
    }
}
pub use apis::*;
