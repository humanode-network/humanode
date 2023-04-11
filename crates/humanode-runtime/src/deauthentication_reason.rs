use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::prelude::*;

#[derive(Clone, PartialEq, Debug, Encode, Decode, TypeInfo)]
pub enum DeauthenticationReason {
    Offence,
}
