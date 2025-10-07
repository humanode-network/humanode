//! The weights.

use frame_support::dispatch::Weight;

/// The weight information trait, to be implemented in from the benches.
pub trait WeightInfo {
    /// Weight for the `update_set` call.
    fn update_set() -> Weight;
}

impl WeightInfo for () {
    fn update_set() -> Weight {
        Weight::zero()
    }
}
