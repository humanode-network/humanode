//! Weights definition for pallet-currency-swap.

use frame_support::weights::Weight;

/// Weight functions needed for pallet-currency-swap.
pub trait WeightInfo {
    /// A function to calculate required weights for swap call.
    fn swap() -> Weight;
}

impl WeightInfo for () {
    fn swap() -> Weight {
        Weight::zero()
    }
}
