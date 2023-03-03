
//! Autogenerated weights for `pallet_bioauth`
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

/// Weight functions for `pallet_bioauth`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bioauth::WeightInfo for WeightInfo<T> {
	/// The range of component `a` is `[0, 3071]`.
	/// The range of component `n` is `[0, 30719999]`.
	fn authenticate(_a: u32, n: u32, ) -> Weight {
		// Minimum execution time: 127_000 nanoseconds.
		Weight::from_ref_time(227_889_999_931)
			// Standard Error: 9_401
			.saturating_add(Weight::from_ref_time(132_701).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// The range of component `a` is `[0, 3072]`.
	fn set_robonode_public_key(_a: u32, ) -> Weight {
		// Minimum execution time: 6_000 nanoseconds.
		Weight::from_ref_time(7_000_000)
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// The range of component `a` is `[0, 3072]`.
	fn on_initialize(_a: u32, ) -> Weight {
		// Minimum execution time: 6_000 nanoseconds.
		Weight::from_ref_time(74_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
