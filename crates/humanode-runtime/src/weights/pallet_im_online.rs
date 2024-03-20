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
  /// The range of component `e` is `[1, 100]`.
  /// The range of component `k` is `[1, 1000]`.
  fn validate_unsigned_and_then_heartbeat(k: u32, e: u32, ) -> Weight {
    // Minimum execution time: 94_000 nanoseconds.
    Weight::from_parts(71_732_732, 0)
      // Standard Error: 5_201
      .saturating_add(Weight::from_parts(45_045, 0).saturating_mul(k.into()))
      // Standard Error: 52_486
      .saturating_add(Weight::from_parts(222_222, 0).saturating_mul(e.into()))
      .saturating_add(T::DbWeight::get().reads(5))
      .saturating_add(T::DbWeight::get().writes(1))
  }
}
