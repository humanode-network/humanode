//! The runtime API for the bioauth consensus.

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

    /// Runtime API for the bioauth consensus with session key.
    pub trait BioauthConsensusSessionApi<Id: Encode> {
        /// Determine whether an [`Id`] is bioauth-authorized at the current block through
        /// a session key ownership.
        fn is_authorized_through_session_key(id: &Id) -> bool;
    }
}
