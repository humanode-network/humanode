//! Weights definition for pallet-bridges-initializer-currency-swap.

use frame_support::weights::Weight;

/// Weight functions needed for pallet-bridges-initializer-currency-swap.
pub trait WeightInfo {
    /// A function to calculate required weights for `verify_balanced` call.
    fn verify_balanced() -> Weight;
}

impl WeightInfo for () {
    fn verify_balanced() -> Weight {
        Weight::zero()
    }
}
