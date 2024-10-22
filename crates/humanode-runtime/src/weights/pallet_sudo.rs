// DO NOT EDIT!
//! Autogenerated weights for `pallet_sudo`

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_sudo`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_sudo::WeightInfo for WeightInfo<T> {
  fn set_key() -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `132`
    //  Estimated: `0`
    // Minimum execution time: 9_000_000 picoseconds.
    Weight::from_parts(9_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(1))
      .saturating_add(T::DbWeight::get().writes(1))
  }
  fn sudo() -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `132`
    //  Estimated: `0`
    // Minimum execution time: 9_000_000 picoseconds.
    Weight::from_parts(9_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(1))
  }
  fn sudo_as() -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `132`
    //  Estimated: `0`
    // Minimum execution time: 9_000_000 picoseconds.
    Weight::from_parts(9_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(1))
  }
}
