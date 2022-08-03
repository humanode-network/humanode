//! Custom types we use.

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::RuntimeDebug;
#[cfg(feature = "std")]
use frame_support::{Deserialize, Serialize};
use primitives_ethereum::EthereumAddress;
use scale_info::TypeInfo;

/// The claim information.
#[derive(
    Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ClaimInfo<Balance, Vesting> {
    /// The amount to claim.
    pub balance: Balance,
    /// The vesting configuration for the given claim.
    pub vesting: Option<Vesting>,
}

/// The collection of parameters used for constructing a message that had to be signed.
#[derive(PartialEq, Eq, RuntimeDebug)]
pub struct EthereumSignatureMessageParams<AccountId> {
    /// The account ID of whoever is requesting the claim.
    pub account_id: AccountId,
    /// The ethereum address the claim is authorized for.
    pub ethereum_address: EthereumAddress,
}
