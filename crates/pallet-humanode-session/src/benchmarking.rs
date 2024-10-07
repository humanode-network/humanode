//! The benchmarks for the pallet.

// Allow integer and float arithmetic in tests.
#![allow(clippy::arithmetic_side_effects, clippy::float_arithmetic)]

pub extern crate alloc;

use alloc::collections::BTreeSet;

use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use sp_runtime::BoundedBTreeSet;

use crate::*;

/// The benchmark interface into the environment.
pub trait Interface: pallet::Config {
    /// Obtain an Account ID based on provided account index.
    fn provide_account_id(account_index: u32) -> <Self as frame_system::Config>::AccountId;
}

/// Populate the [`BannedAccounts`] storage with generated data.
fn populate_banned_accounts<T: Interface>(count: u32) {
    let mut banned_accounts = BTreeSet::new();

    for i in 0..count {
        banned_accounts.insert(T::provide_account_id(i));
    }

    let bounded_banned_accounts =
        BoundedBTreeSet::<_, T::MaxBannedAccounts>::try_from(banned_accounts).unwrap();

    BannedAccounts::<T>::put(bounded_banned_accounts);
}

benchmarks! {
    where_clause {
        where
            T: Interface,
    }

    ban {
        // Vary the amount of pre-populated banned accounts.
        let b in 0 .. (T::MaxBannedAccounts::get() - 1) =>  populate_banned_accounts::<T>(b);

        let account_to_be_banned = T::provide_account_id(T::MaxBannedAccounts::get());

        // Capture some data used during the verification.
        let banned_accounts_before_len = BannedAccounts::<T>::get().len();

    }: _(RawOrigin::Root, account_to_be_banned.clone())
    verify {
        let banned_accounts_after = BannedAccounts::<T>::get();
        // Verify that account was banned.
        assert!(banned_accounts_after.contains(&account_to_be_banned));
        // Verify that exactly one banned account was added.
        assert_eq!(banned_accounts_after.len() - banned_accounts_before_len, 1);
    }

    unban {
        // Vary the amount of pre-populated banned accounts.
        let b in 1 .. (T::MaxBannedAccounts::get()) =>  populate_banned_accounts::<T>(b);

        let account_to_be_unbanned = T::provide_account_id(0);

        // Capture some data used during the verification.
        let banned_accounts_before_len = BannedAccounts::<T>::get().len();

    }: _(RawOrigin::Root, account_to_be_unbanned.clone())
    verify {
        let banned_accounts_after = BannedAccounts::<T>::get();
        // Verify that account was unbanned.
        assert!(!banned_accounts_after.contains(&account_to_be_unbanned));
        // Verify that exactly one banned account was unbanned.
        assert_eq!(banned_accounts_before_len - banned_accounts_after.len(), 1);
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test,
    );
}

#[cfg(test)]
impl Interface for crate::mock::Test {
    fn provide_account_id(account_index: u32) -> <Self as frame_system::Config>::AccountId {
        account_index.into()
    }
}
