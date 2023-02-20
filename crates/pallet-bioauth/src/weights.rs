//! Weights definition for pallet-bioauth.

use frame_support::weights::Weight;

/// Weight functions needed for pallet-bioauth.
pub trait WeightInfo {
    /// A function to calculate required weights for authenticate call.
    fn authenticate() -> Weight;
    /// A function to calculate required weights for `set_robonode_public_key` call.
    fn set_robonode_public_key() -> Weight;
    /// A function to calculate required weights for `on_initialize` hook.
    fn on_initialize() -> Weight;
}

impl WeightInfo for () {
    fn authenticate() -> Weight {
        Weight::zero()
    }

    fn set_robonode_public_key() -> Weight {
        Weight::zero()
    }

    fn on_initialize() -> Weight {
        Weight::zero()
    }
}
