// Allow simple integer arithmetic in tests.
#![allow(clippy::integer_arithmetic)]

#[cfg(feature = "runtime-benchmarks")]
pub(crate) mod benchmarking;

pub(crate) mod testing;
