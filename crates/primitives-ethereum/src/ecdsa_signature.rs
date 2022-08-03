//! ECDSA Signature.

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;

/// A ECDSA signature, used by Ethereum.
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct EcdsaSignature(pub [u8; 65]);

impl Default for EcdsaSignature {
    fn default() -> Self {
        Self([0; 65])
    }
}
