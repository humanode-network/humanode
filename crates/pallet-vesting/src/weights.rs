//! The weights.

use frame_support::dispatch::Weight;

/// The weight information trait, to be implemented in from the benches.
pub trait WeightInfo {
    /// Weight for the unlock call.
    fn unlock() -> Weight;
}

impl WeightInfo for () {
    fn unlock() -> Weight {
        Weight::zero()
    }
}
