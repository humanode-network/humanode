// DO NOT EDIT!
//! Autogenerated weights for `pallet_im_online`

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_im_online`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_im_online::WeightInfo for WeightInfo<T> {
  /// The range of component `k` is `[1, 1000]`.
  /// The range of component `e` is `[1, 100]`.
  fn validate_unsigned_and_then_heartbeat(k: u32, e: u32, ) -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `260 + k * (32 ±0)`
    //  Estimated: `0`
    // Minimum execution time: 85_000_000 picoseconds.
    Weight::from_parts(49_602_420, 0)
      // Standard Error: 8_668
      .saturating_add(Weight::from_parts(44_044, 0).saturating_mul(k.into()))
      // Standard Error: 87_477
      .saturating_add(Weight::from_parts(353_535, 0).saturating_mul(e.into()))
      .saturating_add(T::DbWeight::get().reads(5))
      .saturating_add(T::DbWeight::get().writes(1))
  }
}
