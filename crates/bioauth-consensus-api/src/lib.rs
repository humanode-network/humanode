//! The runtime API for the bioauth consensus.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]
#![cfg_attr(not(feature = "std"), no_std)]
// `decl_runtime_apis` macro has issues.
#![allow(clippy::too_many_arguments, clippy::unnecessary_mut_passed)]

use codec::Encode;
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
    /// Runtime API for the bioauth consensus.
    pub trait BioauthConsensusApi<Id: Encode> {
        /// Determine whether an id is bioauth-authorized at the current block.
        fn is_authorized(id: &Id) -> bool;
    }
}
