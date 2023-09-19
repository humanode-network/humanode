//! The mocks for the pallet testing.

use frame_support::{sp_io, sp_runtime::BuildStorage};
use sp_core::H160;

pub mod v0;
pub mod v1;

pub(crate) const EXISTENTIAL_DEPOSIT: u64 = 10;

pub(crate) type AccountId = u64;
pub(crate) type EvmAccountId = H160;
type Balance = u64;

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
