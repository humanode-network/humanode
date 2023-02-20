//! The weights.

use frame_support::dispatch::Weight;

/// The weight information trait, to be implemented in from the benches.
pub trait WeightInfo {
    /// Weight for `claim` call.
    fn claim() -> Weight;

    /// Weight for `add_claim` call.
    fn add_claim() -> Weight;

    /// Weight for `remove_claim` call.
    fn remove_claim() -> Weight;

    /// Weight for `change_claim` call.
    fn change_claim() -> Weight;
}

impl WeightInfo for () {
    fn claim() -> Weight {
        Weight::zero()
    }

    fn add_claim() -> Weight {
        Weight::zero()
    }

    fn remove_claim() -> Weight {
        Weight::zero()
    }

    fn change_claim() -> Weight {
        Weight::zero()
    }
}
