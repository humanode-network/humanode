//! Weights definition for pallet-humanode-session.

use frame_support::weights::Weight;

/// Weight functions needed for pallet-humanode-session.
pub trait WeightInfo {
    /// A function to calculate required weights for ban call.
    fn ban() -> Weight;

    /// A function to calculate required weights for unban call.
    fn unban() -> Weight;
}

impl WeightInfo for () {
    fn ban() -> Weight {
        Weight::zero()
    }

    fn unban() -> Weight {
        Weight::zero()
    }
}
