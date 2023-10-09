// Allow simple integer arithmetic in tests.
#![allow(clippy::arithmetic_side_effects)]

#[cfg(feature = "runtime-benchmarks")]
pub(crate) mod benchmarking;

pub(crate) mod testing;
