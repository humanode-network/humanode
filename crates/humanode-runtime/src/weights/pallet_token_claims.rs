// DO NOT EDIT!
//! Autogenerated weights for `pallet_token_claims`

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_token_claims`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_token_claims::WeightInfo for WeightInfo<T> {
  fn claim() -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `593`
    //  Estimated: `0`
    // Minimum execution time: 88_000_000 picoseconds.
    Weight::from_parts(88_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(9))
      .saturating_add(T::DbWeight::get().writes(5))
  }
  fn add_claim() -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `264`
    //  Estimated: `0`
    // Minimum execution time: 26_000_000 picoseconds.
    Weight::from_parts(26_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(3))
      .saturating_add(T::DbWeight::get().writes(4))
  }
  fn remove_claim() -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `264`
    //  Estimated: `0`
    // Minimum execution time: 27_000_000 picoseconds.
    Weight::from_parts(27_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(3))
      .saturating_add(T::DbWeight::get().writes(4))
  }
  fn change_claim() -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `264`
    //  Estimated: `0`
    // Minimum execution time: 26_000_000 picoseconds.
    Weight::from_parts(26_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(3))
      .saturating_add(T::DbWeight::get().writes(4))
  }
}
