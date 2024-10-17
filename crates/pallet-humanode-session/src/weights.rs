//! Weights definition for pallet-humanode-session.

use frame_support::weights::Weight;

/// Weight functions needed for pallet-humanode-session.
pub trait WeightInfo {
    /// A function to calculate required weights for ban call.
    fn ban(banned_accounts: u32) -> Weight;

    /// A function to calculate required weights for unban call.
    fn unban(banned_accounts: u32) -> Weight;
}

impl WeightInfo for () {
    fn ban(_banned_accounts: u32) -> Weight {
        Weight::zero()
    }

    fn unban(_banned_accounts: u32) -> Weight {
        Weight::zero()
    }
}
