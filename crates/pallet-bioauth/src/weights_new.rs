
//! Autogenerated weights for `pallet_bioauth`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-06-23, STEPS: `1`, REPEAT: 1, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: None, WASM-EXECUTION: Compiled, CHAIN: Some("benchmark"), DB CACHE: 1024

// Executed Command:
// target/debug/humanode-peer
// benchmark
// pallet
// --pallet
// pallet-bioauth
// --extrinsic
// *
// --chain
// benchmark
// --output
// ./crates/pallet-bioauth/src/weights_new.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_bioauth`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bioauth::WeightInfo for WeightInfo<T> {
	// Storage: Bioauth RobonodePublicKey (r:1 w:0)
	// Storage: Bioauth ConsumedAuthTicketNonces (r:1 w:1)
	// Storage: Bioauth ActiveAuthentications (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	fn authenticate(_i: u32, ) -> Weight {
		(73_505_146_961_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Bioauth RobonodePublicKey (r:0 w:1)
	// Storage: Bioauth ActiveAuthentications (r:0 w:1)
	fn set_robonode_public_key() -> Weight {
		(317_102_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Bioauth ActiveAuthentications (r:1 w:1)
	fn on_initialize(_b: u32, ) -> Weight {
		(325_671_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
