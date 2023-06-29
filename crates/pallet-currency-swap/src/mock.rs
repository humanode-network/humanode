//! The mock for the pallet.

// Allow simple integer arithmetic in tests.
#![allow(clippy::integer_arithmetic)]

use frame_support::{
    sp_io,
    sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage, DispatchError,
    },
    traits::{ConstU32, ConstU64, Currency, ExistenceRequirement, Imbalance, WithdrawReasons},
};
use sp_core::{H160, H256};

use crate::{self as pallet_currency_swap, traits};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances,
        EvmSystem: pallet_evm_system,
        EvmBalances: pallet_evm_balances,
        CurrencySwap: pallet_currency_swap,
    }
);

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<u64>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
    type Balance = u64;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<1>;
    type AccountStore = System;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl pallet_evm_system::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = H160;
    type Index = u64;
    type AccountData = pallet_evm_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
}

impl pallet_evm_balances::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = H160;
    type Balance = u64;
    type ExistentialDeposit = ConstU64<1>;
    type AccountStore = EvmSystem;
    type DustRemoval = ();
}

pub struct NativeToEvmOneToOne;

impl traits::CurrencySwap<u64, H160> for NativeToEvmOneToOne {
    type From = Balances;
    type To = EvmBalances;
    type Error = DispatchError;

    fn swap(
        account_id_from: &u64,
        account_id_to: &H160,
        amount: u64,
    ) -> Result<u64, DispatchError> {
        let withdraw_imbalance = Balances::withdraw(
            account_id_from,
            amount,
            WithdrawReasons::TRANSFER,
            ExistenceRequirement::AllowDeath,
        )?;

        let deposit_imbalance =
            EvmBalances::deposit_creating(account_id_to, withdraw_imbalance.peek());

        Ok(deposit_imbalance.peek())
    }
}

impl pallet_currency_swap::Config for Test {
    type AccountIdTo = H160;
    type CurrencySwap = NativeToEvmOneToOne;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let genesis_config = GenesisConfig::default();
    new_test_ext_with(genesis_config)
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext_with(genesis_config: GenesisConfig) -> sp_io::TestExternalities {
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

pub trait TestExternalitiesExt {
    fn execute_with_ext<R, E>(&mut self, execute: E) -> R
    where
        E: for<'e> FnOnce(&'e ()) -> R;
}

impl TestExternalitiesExt for frame_support::sp_io::TestExternalities {
    fn execute_with_ext<R, E>(&mut self, execute: E) -> R
    where
        E: for<'e> FnOnce(&'e ()) -> R,
    {
        let guard = runtime_lock();
        let result = self.execute_with(|| execute(&guard));
        drop(guard);
        result
    }
}
