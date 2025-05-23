// DO NOT EDIT!
//! Autogenerated weights for `pallet_native_to_evm_swap`

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_native_to_evm_swap`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_native_to_evm_swap::WeightInfo for WeightInfo<T> {
  fn swap() -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `995`
    //  Estimated: `0`
    // Minimum execution time: 72_000_000 picoseconds.
    Weight::from_parts(72_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(10))
      .saturating_add(T::DbWeight::get().writes(5))
  }
  fn swap_keep_alive() -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `995`
    //  Estimated: `0`
    // Minimum execution time: 70_000_000 picoseconds.
    Weight::from_parts(70_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(10))
      .saturating_add(T::DbWeight::get().writes(5))
  }
}
