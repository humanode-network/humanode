//! The weights.

use frame_support::dispatch::Weight;

/// The weight information trait, to be implemented from the benches.
pub trait WeightInfo {
    /// Weight for the `set_native_fee_multiplier` call.
    fn set_native_fee_multiplier() -> Weight;

    /// Weight for the `set_evm_fee_multiplier` call.
    fn set_evm_fee_multiplier() -> Weight;
}

impl WeightInfo for () {
    fn set_native_fee_multiplier() -> Weight {
        Weight::zero()
    }

    fn set_evm_fee_multiplier() -> Weight {
        Weight::zero()
    }
}
