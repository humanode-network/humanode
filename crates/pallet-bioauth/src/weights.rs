//! Weights definition for pallet-bioauth.

use frame_support::weights::Weight;

/// Weight functions needed for pallet-bioauth.
pub trait WeightInfo {
    /// A function to calculate required weights for authenticate call.
    fn authenticate(
        authentications: u32,
        black_listed_validators_public_keys: u32,
        nonces: u32,
    ) -> Weight;
    /// A function to calculate required weights for `set_robonode_public_key` call.
    fn set_robonode_public_key(authentications: u32) -> Weight;
    /// A function to calculate required weights for `blacklist` call.
    fn blacklist(black_listed_validators_public_keys: u32) -> Weight;
    /// A function to calculate required weights for `on_initialize` hook.
    fn on_initialize(authentications: u32) -> Weight;
}

impl WeightInfo for () {
    fn authenticate(
        _authentications: u32,
        _black_listed_validators_public_keys: u32,
        _nonces: u32,
    ) -> Weight {
        Weight::zero()
    }

    fn set_robonode_public_key(_authentications: u32) -> Weight {
        Weight::zero()
    }

    fn blacklist(_black_listed_validators_public_keys: u32) -> Weight {
        Weight::zero()
    }

    fn on_initialize(_authentications: u32) -> Weight {
        Weight::zero()
    }
}
