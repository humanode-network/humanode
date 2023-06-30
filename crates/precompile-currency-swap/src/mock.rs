//! The mock for the precompile.

// Allow simple integer arithmetic in tests.
#![allow(clippy::integer_arithmetic)]

use fp_evm::{Context, ExitError, ExitReason, PrecompileHandle, Transfer};
use frame_support::{
    sp_io,
    sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage, DispatchError,
    },
    traits::{ConstU64, Currency},
};
use frame_system as system;
use mockall::{mock, predicate::*, *};
use sp_core::{H160, H256, U256};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub(crate) type AccountId = u64;
pub(crate) type EvmAccountId = H160;
pub(crate) type Balance = u64;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Balances: pallet_balances,
        EvmSystem: pallet_evm_system,
        EvmBalances: pallet_evm_balances,
        EVM: pallet_evm,
    }
);

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

frame_support::parameter_types! {
    pub const MinimumPeriod: u64 = 1000;
}
impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
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
    type AccountId = EvmAccountId;
    type Index = u64;
    type AccountData = pallet_evm_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
}

impl pallet_evm_balances::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = EvmAccountId;
    type Balance = Balance;
    type ExistentialDeposit = ConstU64<1>;
    type AccountStore = EvmSystem;
    type DustRemoval = ();
}

pub struct FixedGasPrice;
impl fp_evm::FeeCalculator for FixedGasPrice {
    fn min_gas_price() -> (U256, Weight) {
        // Return some meaningful gas price and weight
        (1_000_000_000u128.into(), Weight::from_ref_time(7u64))
    }
}

frame_support::parameter_types! {
    pub BlockGasLimit: U256 = U256::max_value();
    pub WeightPerGas: Weight = Weight::from_ref_time(20_000);
    pub MockPrecompiles: MockPrecompileSet = MockPrecompileSet;
}

impl pallet_evm::Config for Test {
    type AccountProvider = pallet_evm::NativeSystemAccountProvider<Self>;
    type FeeCalculator = FixedGasPrice;
    type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
    type WeightPerGas = WeightPerGas;

    type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
    type CallOrigin = pallet_evm::EnsureAddressNever<
        <Self::AccountProvider as pallet_evm::AccountProvider>::AccountId,
    >;
    type WithdrawOrigin = pallet_evm::EnsureAddressNever<
        <Self::AccountProvider as pallet_evm::AccountProvider>::AccountId,
    >;
    type AddressMapping = pallet_evm::IdentityAddressMapping;
    type Currency = Balances;

    type RuntimeEvent = RuntimeEvent;
    type PrecompilesType = MockPrecompileSet;
    type PrecompilesValue = MockPrecompiles;
    type ChainId = ();
    type BlockGasLimit = BlockGasLimit;
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    type OnChargeTransaction = ();
    type OnCreate = ();
    type FindAuthor = ();
}

mock! {
    #[derive(Debug)]
    pub CurrencySwap {}
    impl primitives_currency_swap::CurrencySwap<AccountId, EvmAccountId> for CurrencySwap {
        type From = Balances;
        type To = EvmBalances;
        type Error = DispatchError;

        fn swap(
            imbalance: <Balances as Currency<AccountId>>::NegativeImbalance,
        ) -> Result<<EvmBalances as Currency<EvmAccountId>>::NegativeImbalance, DispatchError>;
    }
}

mock! {
    #[derive(Debug)]
    pub PrecompileHandle {}

    impl PrecompileHandle for PrecompileHandle {
        fn call(
            &mut self,
            to: sp_core::H160,
            transfer: Option<Transfer>,
            input: Vec<u8>,
            gas_limit: Option<u64>,
            is_static: bool,
            context: &Context,
        ) -> (ExitReason, Vec<u8>);

        fn record_cost(&mut self, cost: u64) -> Result<(), ExitError>;

        fn remaining_gas(&self) -> u64;

        fn log(&mut self, address: sp_core::H160, topics: Vec<sp_core::H256>, data: Vec<u8>) -> Result<(), ExitError>;

        fn code_address(&self) -> sp_core::H160;

        fn input(&self) -> &[u8];

        fn context(&self) -> &Context;

        fn is_static(&self) -> bool;

        fn gas_limit(&self) -> Option<u64>;
    }
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
