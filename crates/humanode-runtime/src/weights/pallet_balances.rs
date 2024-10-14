// DO NOT EDIT!
//! Autogenerated weights for `pallet_balances`

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_balances`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_balances::WeightInfo for WeightInfo<T> {
  fn transfer_allow_death() -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `103`
    //  Estimated: `0`
    // Minimum execution time: 37_000_000 picoseconds.
    Weight::from_parts(37_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(2))
      .saturating_add(T::DbWeight::get().writes(2))
  }
  fn transfer_keep_alive() -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `0`
    //  Estimated: `0`
    // Minimum execution time: 22_000_000 picoseconds.
    Weight::from_parts(22_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(1))
      .saturating_add(T::DbWeight::get().writes(1))
  }
  fn force_set_balance_creating() -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `103`
    //  Estimated: `0`
    // Minimum execution time: 8_000_000 picoseconds.
    Weight::from_parts(8_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(1))
      .saturating_add(T::DbWeight::get().writes(1))
  }
  fn force_set_balance_killing() -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `103`
    //  Estimated: `0`
    // Minimum execution time: 12_000_000 picoseconds.
    Weight::from_parts(12_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(1))
      .saturating_add(T::DbWeight::get().writes(1))
  }
  fn force_transfer() -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `242`
    //  Estimated: `0`
    // Minimum execution time: 37_000_000 picoseconds.
    Weight::from_parts(37_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(3))
      .saturating_add(T::DbWeight::get().writes(3))
  }
  fn transfer_all() -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `0`
    //  Estimated: `0`
    // Minimum execution time: 24_000_000 picoseconds.
    Weight::from_parts(24_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(1))
      .saturating_add(T::DbWeight::get().writes(1))
  }
  fn force_unreserve() -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `103`
    //  Estimated: `0`
    // Minimum execution time: 10_000_000 picoseconds.
    Weight::from_parts(10_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(1))
      .saturating_add(T::DbWeight::get().writes(1))
  }
  /// The range of component `u` is `[1, 1000]`.
  fn upgrade_accounts(_u: u32, ) -> Weight {
    // Proof Size summary in bytes:
    //  Measured:  `0 + u * (135 ±0)`
    //  Estimated: `0`
    // Minimum execution time: 11_000_000 picoseconds.
    Weight::from_parts(8_118_000_000, 0)
      .saturating_add(T::DbWeight::get().reads(1000))
      .saturating_add(T::DbWeight::get().writes(1000))
  }
}
