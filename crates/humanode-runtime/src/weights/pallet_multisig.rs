
//! Autogenerated weights for `pallet_multisig`
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

/// Weight functions for `pallet_multisig`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_multisig::WeightInfo for WeightInfo<T> {
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_threshold_1(_z: u32, ) -> Weight {
		// Minimum execution time: 12_000 nanoseconds.
		Weight::from_ref_time(14_000_000)
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	/// The range of component `s` is `[2, 128]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_create(s: u32, z: u32, ) -> Weight {
		// Minimum execution time: 36_000 nanoseconds.
		Weight::from_ref_time(30_920_634)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(39_682).saturating_mul(s.into()))
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(700).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	/// The range of component `s` is `[3, 128]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_approve(s: u32, z: u32, ) -> Weight {
		// Minimum execution time: 24_000 nanoseconds.
		Weight::from_ref_time(19_904_000)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(32_000).saturating_mul(s.into()))
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(1_100).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	/// The range of component `s` is `[2, 128]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_complete(s: u32, z: u32, ) -> Weight {
		// Minimum execution time: 34_000 nanoseconds.
		Weight::from_ref_time(26_380_952)
			// Standard Error: 6_873
			.saturating_add(Weight::from_ref_time(59_523).saturating_mul(s.into()))
			// Standard Error: 86
			.saturating_add(Weight::from_ref_time(1_150).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	/// The range of component `s` is `[2, 128]`.
	fn approve_as_multi_create(_s: u32, ) -> Weight {
		// Minimum execution time: 29_000 nanoseconds.
		Weight::from_ref_time(31_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	/// The range of component `s` is `[2, 128]`.
	fn approve_as_multi_approve(_s: u32, ) -> Weight {
		// Minimum execution time: 18_000 nanoseconds.
		Weight::from_ref_time(23_000_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	/// The range of component `s` is `[2, 128]`.
	fn cancel_as_multi(_s: u32, ) -> Weight {
		// Minimum execution time: 27_000 nanoseconds.
		Weight::from_ref_time(30_000_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
