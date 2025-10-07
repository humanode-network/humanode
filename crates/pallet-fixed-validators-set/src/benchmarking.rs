//! The benchmarks for the pallet.

// Allow integer and float arithmetic in tests.
#![allow(clippy::arithmetic_side_effects, clippy::float_arithmetic)]

use frame_benchmarking::benchmarks;
use frame_support::pallet_prelude::*;
use frame_system::RawOrigin;

use crate::*;

/// The benchmark interface into the environment.
pub trait Interface: super::Config {
    /// Provide a fixed amount of validator IDs.
    ///
    /// This is used for benchmarking the set update.
    fn provide_validator_id(index: u32) -> <Self as Config>::ValidatorId;
}

benchmarks! {
    where_clause {
        where
            T: Interface,
    }

    update_set {
        use frame_support::traits::TryCollect as _;
        let new_set = (0..<T as Config>::MaxValidators::get())
            .map(|index| <T as Interface>::provide_validator_id(index))
            .try_collect()
            .unwrap();

    }: _(RawOrigin::Root, new_set.clone())
    verify {
        assert_eq!(Validators::<T>::get(), new_set);
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test,
    );
}

#[cfg(test)]
impl Interface for crate::mock::Test {
    fn provide_validator_id(index: u32) -> <Self as Config>::ValidatorId {
        index.into()
    }
}
