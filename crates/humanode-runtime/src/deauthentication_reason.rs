//! `DeauthenticationReason` implementation to define reasons by which authentication is removed
//! before expected time expiration.

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::prelude::*;

/// Define a possible deauthentication reason.
#[derive(Clone, PartialEq, Debug, Encode, Decode, TypeInfo)]
pub enum DeauthenticationReason {
    /// Some offence has been recevied.
    Offence,
}
