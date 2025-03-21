// DO NOT EDIT!
//! Autogenerated weights for `pallet_humanode_session`

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_humanode_session`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_humanode_session::WeightInfo for WeightInfo<T> {
  /// The range of component `b` is `[0, 3071]`.
  fn ban(_b: u32, ) -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `233 + b * (32 ±0)`
    //  Estimated: `0`
    // Minimum execution time: 10_000_000 picoseconds.
    Weight::from_parts(95_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(2))
      .saturating_add(T::DbWeight::get().writes(1))
  }
  /// The range of component `b` is `[1, 3072]`.
  fn unban(_b: u32, ) -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `171 + b * (32 ±0)`
    //  Estimated: `0`
    // Minimum execution time: 7_000_000 picoseconds.
    Weight::from_parts(92_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(1))
      .saturating_add(T::DbWeight::get().writes(1))
  }
}
