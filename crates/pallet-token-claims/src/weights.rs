//! The weights.

use frame_support::dispatch::Weight;

/// The weight information trait, to be implemented in from the benches.
pub trait WeightInfo {
    /// Weight for `claim` call.
    fn claim() -> Weight;
}

impl WeightInfo for () {
    fn claim() -> Weight {
        Weight::zero()
    }
}
