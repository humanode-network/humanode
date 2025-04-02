//! The benchmarks for the pallet.

// Allow integer and float arithmetic in tests.
#![allow(clippy::arithmetic_side_effects, clippy::float_arithmetic)]

use frame_benchmarking::benchmarks;
use frame_support::{
    assert_ok, dispatch::DispatchResult, sp_runtime::traits::Convert, traits::fungible::Unbalanced,
};
use frame_system::RawOrigin;

use crate::*;

/// The benchmark interface into the environment.
pub trait Interface: super::Config {
    /// The data to be passed from `prepare` to `verify`.
    type Data;

    /// Prepare environment.
    fn prepare() -> Self::Data;

    /// Verify environment,
    fn verify(data: Self::Data) -> DispatchResult;

    /// Obtain the native Account ID the balance is swapped from.
    fn from_native_account_id() -> <Self as frame_system::Config>::AccountId;

    /// Obtain the EVM Account ID the balance is swapped to.
    fn to_evm_account_id() -> <Self as Config>::EvmAccountId;

    /// Obtain the amount of balance to be swapped.
    fn swap_balance() -> NativeBalanceOf<Self>;
}

benchmarks! {
    where_clause {
        where
            T: Interface,
    }

    swap {
        let (origin, swap_data) = prepare_swap_data::<T>();
    }: _(origin, swap_data.to_evm_account.clone(), swap_data.swap_balance)
    verify {
        verify_swap_data::<T>(swap_data);
    }

    swap_keep_alive {
        let (origin, swap_data) = prepare_swap_data::<T>();
    }: _(origin, swap_data.to_evm_account.clone(), swap_data.swap_balance)
    verify {
        verify_swap_data::<T>(swap_data);
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test,
    );
}

/// A helper struct used for preparing and verifying swap calls.
struct SwapData<T: Interface> {
    /// The native Account ID the balance is swapped from.
    from_native_account: <T as frame_system::Config>::AccountId,
    /// The native Account ID balance before executing the call.
    from_native_balance_before: NativeBalanceOf<T>,
    /// The EVM Account ID the balance is swapped to.
    to_evm_account: <T as Config>::EvmAccountId,
    /// The EVM Account ID balance before executing the call.
    to_evm_balance_before: EvmBalanceOf<T>,
    /// The amount of balance to be swapped.
    swap_balance: NativeBalanceOf<T>,
    /// Environment data.
    env_data: T::Data,
}

/// Prepare swap data before executing the corresponding call.
fn prepare_swap_data<T: Interface>() -> (
    RawOrigin<<T as frame_system::Config>::AccountId>,
    SwapData<T>,
) {
    let from_native_account = <T as Interface>::from_native_account_id();
    let to_evm_account = <T as Interface>::to_evm_account_id();
    let swap_balance = <T as Interface>::swap_balance();
    let init_balance: u32 = 1000;

    let _ = T::NativeToken::write_balance(&from_native_account, init_balance.into()).unwrap();

    let from_native_balance_before = T::NativeToken::total_balance(&from_native_account);
    let to_evm_balance_before = T::EvmToken::total_balance(&to_evm_account);

    let env_data = <T as Interface>::prepare();

    let origin = RawOrigin::Signed(from_native_account.clone());

    (
        origin,
        SwapData {
            from_native_account,
            from_native_balance_before,
            to_evm_account,
            to_evm_balance_before,
            swap_balance,
            env_data,
        },
    )
}

/// Verify swap data after executing the corresponding call.
fn verify_swap_data<T: Interface>(swap_data: SwapData<T>) {
    let SwapData {
        from_native_account,
        from_native_balance_before,
        to_evm_account,
        to_evm_balance_before,
        swap_balance,
        env_data,
    } = swap_data;

    let estimated_swapped_balance = T::BalanceConverterNativeToEvm::convert(swap_balance);
    let from_native_balance_after = T::NativeToken::total_balance(&from_native_account);
    let to_evm_balance_after = T::EvmToken::total_balance(&to_evm_account);

    assert_eq!(
        from_native_balance_before - from_native_balance_after,
        swap_balance
    );
    assert_eq!(
        to_evm_balance_after - to_evm_balance_before,
        estimated_swapped_balance
    );

    assert_ok!(<T as Interface>::verify(env_data));
}

#[cfg(test)]
impl Interface for crate::mock::Test {
    type Data = std::sync::MutexGuard<'static, ()>;

    fn prepare() -> Self::Data {
        mock::runtime_lock()
    }

    fn verify(data: Self::Data) -> DispatchResult {
        let mock_runtime_guard = data;
        drop(mock_runtime_guard);
        Ok(())
    }

    fn from_native_account_id() -> <Self as frame_system::Config>::AccountId {
        mock::AccountId::from(hex_literal::hex!(
            "7000000000000000000000000000000000000000000000000000000000000007"
        ))
    }

    fn to_evm_account_id() -> <Self as Config>::EvmAccountId {
        mock::EvmAccountId::from(hex_literal::hex!(
            "7000000000000000000000000000000000000007"
        ))
    }

    fn swap_balance() -> NativeBalanceOf<Self> {
        100
    }
}
