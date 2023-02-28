
//! Autogenerated weights for `pallet_token_claims`
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

/// Weight functions for `pallet_token_claims`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_token_claims::WeightInfo for WeightInfo<T> {
	// Storage: System BlockHash (r:1 w:0)
	// Storage: TokenClaims Claims (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: Vesting Schedules (r:1 w:0)
	// Storage: ChainStartMoment ChainStart (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Balances Locks (r:1 w:1)
	// Storage: TokenClaims TotalClaimable (r:0 w:1)
	fn claim() -> Weight {
		// Minimum execution time: 94_000 nanoseconds.
		Weight::from_ref_time(94_000_000)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: TokenClaims Claims (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: TokenClaims TotalClaimable (r:0 w:1)
	fn add_claim() -> Weight {
		// Minimum execution time: 32_000 nanoseconds.
		Weight::from_ref_time(32_000_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: TokenClaims Claims (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: TokenClaims TotalClaimable (r:0 w:1)
	fn remove_claim() -> Weight {
		// Minimum execution time: 31_000 nanoseconds.
		Weight::from_ref_time(31_000_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: TokenClaims Claims (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: TokenClaims TotalClaimable (r:0 w:1)
	fn change_claim() -> Weight {
		// Minimum execution time: 32_000 nanoseconds.
		Weight::from_ref_time(32_000_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(4))
	}
}
