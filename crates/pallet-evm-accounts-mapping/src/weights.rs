//! The weights.

use frame_support::dispatch::Weight;

/// The weight information trait, to be implemented in from the benches.
pub trait WeightInfo {
    /// Weight for `claim_account` call.
    fn claim_account() -> Weight;
}

impl WeightInfo for () {
    fn claim_account() -> Weight {
        Weight::zero()
    }
}
