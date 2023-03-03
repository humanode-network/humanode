
//! Autogenerated weights for `frame_system`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-03-03, STEPS: `2`, REPEAT: 1, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `mozgiii-mba`, CPU: `<UNKNOWN>`
//! EXECUTION: None, WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// target/release/humanode-peer
// benchmark
// pallet
// --json-input
// crates/humanode-runtime/assets/benchmark.json
// --output
// crates/humanode-runtime/src/weights

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `frame_system`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> frame_system::WeightInfo for WeightInfo<T> {
	/// The range of component `b` is `[0, 3932160]`.
	fn remark(_b: u32, ) -> Weight {
		// Minimum execution time: 3_000 nanoseconds.
		Weight::from_ref_time(336_000_000)
	}
	/// The range of component `b` is `[0, 3932160]`.
	fn remark_with_event(_b: u32, ) -> Weight {
		// Minimum execution time: 12_000 nanoseconds.
		Weight::from_ref_time(4_281_000_000)
	}
	fn set_heap_pages() -> Weight {
		// Minimum execution time: 8_000 nanoseconds.
		Weight::from_ref_time(8_000_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// The range of component `i` is `[0, 1000]`.
	fn set_storage(_i: u32, ) -> Weight {
		// Minimum execution time: 4_000 nanoseconds.
		Weight::from_ref_time(632_000_000)
			.saturating_add(T::DbWeight::get().writes(1000))
	}
	/// The range of component `i` is `[0, 1000]`.
	fn kill_storage(_i: u32, ) -> Weight {
		// Minimum execution time: 3_000 nanoseconds.
		Weight::from_ref_time(580_000_000)
			.saturating_add(T::DbWeight::get().writes(1000))
	}
	/// The range of component `p` is `[0, 1000]`.
	fn kill_prefix(_p: u32, ) -> Weight {
		// Minimum execution time: 7_000 nanoseconds.
		Weight::from_ref_time(1_467_000_000)
			.saturating_add(T::DbWeight::get().writes(1000))
	}
}
