//! The weights.

use frame_support::dispatch::Weight;

/// The weight information trait, to be implemented in from the benches.
pub trait WeightInfo {
    /// Weight for the `unlock` call.
    fn unlock() -> Weight;

    /// Weight for the `update_schedule` call.
    fn update_schedule() -> Weight;
}

impl WeightInfo for () {
    fn unlock() -> Weight {
        Weight::zero()
    }

    fn update_schedule() -> Weight {
        Weight::zero()
    }
}
