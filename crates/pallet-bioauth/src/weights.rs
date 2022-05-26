//! Weights definition for pallet-bioauth.

use frame_support::{traits::DefensiveSaturating, traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet-bioauth.
pub trait WeightInfo {
    /// A function to calculate required weights for authenticate call.
    fn authenticate() -> Weight;
    /// A function to calculate required weights for on_initialize hook.
    fn on_initialize(update_required: bool) -> Weight;
}

/// A helper function to calculate weights.
pub fn calculate_weight<T: frame_system::Config>(
    start_weight: Weight,
    reads: Weight,
    writes: Weight,
) -> Weight {
    start_weight
        .defensive_saturating_add(T::DbWeight::get().reads(reads))
        .defensive_saturating_add(T::DbWeight::get().writes(writes))
}

/// Weights for pallet-bioauth using the Humanode Substrate-based node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    // Storage: Bioauth RobonodePublicKey (r:1 w:0)
    // Storage: Bioauth ConsumedAuthTicketNonces (r:1 w:1)
    // Storage: Bioauth ActiveAuthentications (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Bioauth AuthenticationsExpireAfter (r:1 w:0)
    // Storage: Authorities (r:1 w:1)
    fn authenticate() -> Weight {
        calculate_weight::<T>(10_000_u64, 6_u64, 3_u64)
    }

    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Bioauth ActiveAuthentications (r:1 w:0)
    // Storage: Bioauth ActiveAuthentications (r:0 w:1) if update_required
    // Storage: Authorities (r:1 w:1) if update_required
    fn on_initialize(update_required: bool) -> Weight {
        if update_required {
            calculate_weight::<T>(10_000_u64, 3_u64, 2_u64)
        } else {
            calculate_weight::<T>(10_000_u64, 2_u64, 0_u64)
        }
    }
}
