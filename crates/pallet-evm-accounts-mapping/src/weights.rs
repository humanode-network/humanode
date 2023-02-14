//! The weights.

use frame_support::{dispatch::Weight, traits::Get};
use sp_std::marker::PhantomData;

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

/// A helper function to calculate weights.
pub fn calculate_weight<T: frame_system::Config>(
    start_weight: Weight,
    reads: u64,
    writes: u64,
) -> Weight {
    start_weight
        .saturating_add(T::DbWeight::get().reads(reads))
        .saturating_add(T::DbWeight::get().writes(writes))
}

/// Weights for pallet-evm-accounts-mapping using the Humanode Substrate-based node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    // Storage: EvmAccountsMapping EthereumAddresses (r:1 w:1)
    // Storage: EvmAccountsMapping Accounts (r:1 w:1)
    fn claim_account() -> Weight {
        calculate_weight::<T>(Weight::from_ref_time(10_000_u64), 2_u64, 2_u64)
    }
}
