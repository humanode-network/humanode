//! The mocks for the pallet testing.

use frame_support::{parameter_types, sp_io, sp_runtime::BuildStorage};

use crate::{self as pallet_balanced_currency_swap_bridges_initializer};

pub mod v0;
pub mod v1;
pub mod v2;

pub(crate) const EXISTENTIAL_DEPOSIT_NATIVE: u64 = 10;
pub(crate) const EXISTENTIAL_DEPOSIT_EVM: u64 = 20;
pub(crate) const EXISTENTIAL_DEPOSIT_EVM_NEW: u64 = 1;

pub(crate) type AccountId = u64;
pub(crate) type EvmAccountId = u64;

type Balance = u64;

type BalancesInstanceNative = pallet_balances::Instance1;
type BalancesInstanceEvm = pallet_balances::Instance2;

parameter_types! {
    pub NativeTreasury: AccountId = 4200;
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext_with(genesis_config: impl BuildStorage) -> sp_io::TestExternalities {
    let storage = genesis_config.build_storage().unwrap();
    storage.into()
}

pub fn runtime_lock() -> std::sync::MutexGuard<'static, ()> {
    static MOCK_RUNTIME_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    // Ignore the poisoning for the tests that panic.
    // We only care about concurrency here, not about the poisoning.
    match MOCK_RUNTIME_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub fn with_runtime_lock<R>(f: impl FnOnce() -> R) -> R {
    let lock = runtime_lock();
    let res = f();
    drop(lock);
    res
}

#[derive(Clone, Copy)]
pub(crate) struct AccountInfo {
    pub account: u64,
    pub balance: u64,
}

impl From<AccountInfo> for (u64, u64) {
    fn from(account_info: AccountInfo) -> Self {
        (account_info.account, account_info.balance)
    }
}

pub(crate) const ALICE: AccountInfo = AccountInfo {
    account: 4201,
    balance: 20,
};

pub(crate) const BOB: AccountInfo = AccountInfo {
    account: 4202,
    balance: 30,
};

pub(crate) const LION: AccountInfo = AccountInfo {
    account: 4211,
    balance: 200,
};

pub(crate) const DOG: AccountInfo = AccountInfo {
    account: 4212,
    balance: 300,
};

pub(crate) const CAT: AccountInfo = AccountInfo {
    account: 4213,
    balance: 400,
};

pub(crate) const FISH: AccountInfo = AccountInfo {
    account: 4214,
    balance: 500,
};
