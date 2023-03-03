//! The benchmarks for the pallet.

use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;

use crate::*;

benchmarks! {
    update_inactive {


    }: _(origin, ethereum_address, ethereum_signature)
    verify {

    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test,
    );
}
