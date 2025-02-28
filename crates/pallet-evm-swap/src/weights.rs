//! Weights definition for pallet-native-to-evm-currency-swap.

use frame_support::weights::Weight;

/// Weight functions needed for pallet-currency-swap.
pub trait WeightInfo {
    /// A function to calculate required weights for swap call.
    fn swap() -> Weight;

    /// A function to calculate required weights for `swap_keep_alive` call.
    fn swap_keep_alive() -> Weight;
}

impl WeightInfo for () {
    fn swap() -> Weight {
        Weight::zero()
    }

    fn swap_keep_alive() -> Weight {
        Weight::zero()
    }
}
