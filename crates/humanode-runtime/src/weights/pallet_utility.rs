
//! Autogenerated weights for `pallet_utility`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-28, STEPS: `2`, REPEAT: 1, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `Dmitrys-MacBook-Air.local`, CPU: `<UNKNOWN>`
//! EXECUTION: None, WASM-EXECUTION: Compiled, CHAIN: Some("benchmark"), DB CACHE: 1024

// Executed Command:
// ./target/release/humanode-peer
// benchmark
// pallet
// --chain
// benchmark
// --pallet
// *
// --extrinsic
// *
// --output
// ./crates/humanode-runtime/src/weights

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_utility`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_utility::WeightInfo for WeightInfo<T> {
	/// The range of component `c` is `[0, 1000]`.
	fn batch(_c: u32, ) -> Weight {
		// Minimum execution time: 11_000 nanoseconds.
		Weight::from_ref_time(1_926_000_000)
	}
	fn as_derivative() -> Weight {
		// Minimum execution time: 7_000 nanoseconds.
		Weight::from_ref_time(7_000_000)
	}
	/// The range of component `c` is `[0, 1000]`.
	fn batch_all(_c: u32, ) -> Weight {
		// Minimum execution time: 11_000 nanoseconds.
		Weight::from_ref_time(1_949_000_000)
	}
	fn dispatch_as() -> Weight {
		// Minimum execution time: 13_000 nanoseconds.
		Weight::from_ref_time(13_000_000)
	}
	/// The range of component `c` is `[0, 1000]`.
	fn force_batch(_c: u32, ) -> Weight {
		// Minimum execution time: 11_000 nanoseconds.
		Weight::from_ref_time(1_880_000_000)
	}
}