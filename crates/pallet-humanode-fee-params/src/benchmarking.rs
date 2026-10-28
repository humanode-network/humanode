//! The benchmarks for the pallet.

// Allow integer and float arithmetic in tests.
#![allow(clippy::float_arithmetic)]

use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;

use crate::*;

benchmarks! {
    set_native_fee_multiplier {
        let multiplier = FixedU128::from_u32(123);
    }: _(RawOrigin::Root, multiplier)
    verify {
        assert_eq!(NativeFeeMultiplier::<T>::get(), multiplier);
    }

    set_evm_fee_multiplier {
        let multiplier = FixedU128::from_u32(123);
    }: _(RawOrigin::Root, multiplier)
    verify {
        assert_eq!(EvmFeeMultiplier::<T>::get(), multiplier);
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test,
    );
}
