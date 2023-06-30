//! The mock for the precompile.

// Allow simple integer arithmetic in tests.
#![allow(clippy::integer_arithmetic)]

use fp_evm::{Context, ExitError, ExitReason, PrecompileHandle, Transfer};
use frame_support::{
    once_cell::sync::Lazy,
    sp_io,
    sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage, DispatchError,
    },
    traits::{ConstU16, ConstU32, ConstU64, Currency},
    weights::Weight,
};
use frame_system as system;
use mockall::mock;
use sp_core::{ConstU128, H160, H256, U256};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub(crate) type AccountId = sp_runtime::AccountId32;
pub(crate) type EvmAccountId = H160;
pub(crate) type Balance = u128;

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
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<1>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
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
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<1>;
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
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = EvmSystem;
    type DustRemoval = ();
}

pub(crate) static GAS_PRICE: Lazy<U256> = Lazy::new(|| 1_000_000_000u128.into());

pub struct FixedGasPrice;
impl fp_evm::FeeCalculator for FixedGasPrice {
    fn min_gas_price() -> (U256, Weight) {
        // Return some meaningful gas price and weight
        (*GAS_PRICE, Weight::from_ref_time(7u64))
    }
}

frame_support::parameter_types! {
    pub BlockGasLimit: U256 = U256::max_value();
    pub WeightPerGas: Weight = Weight::from_ref_time(20_000);
    pub MockPrecompiles: MockPrecompileSet = MockPrecompileSet;
}

impl pallet_evm::Config for Test {
    type AccountProvider = EvmSystem;
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
    type Currency = EvmBalances;
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

type CurrencySwapPrecompile =
    crate::CurrencySwap<MockCurrencySwap, EvmAccountId, AccountId, ConstU64<200>>;

/// The precompile set containing the precompile under test.
pub struct MockPrecompileSet;

pub(crate) static PRECOMPILE_ADDRESS: Lazy<H160> = Lazy::new(|| H160::from_low_u64_be(0x900));

impl pallet_evm::PrecompileSet for MockPrecompileSet {
    /// Tries to execute a precompile in the precompile set.
    /// If the provided address is not a precompile, returns None.
    fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<pallet_evm::PrecompileResult> {
        use pallet_evm::Precompile;
        let address = handle.code_address();

        if address == *PRECOMPILE_ADDRESS {
            return Some(CurrencySwapPrecompile::execute(handle));
        }

        None
    }

    /// Check if the given address is a precompile. Should only be called to
    /// perform the check while not executing the precompile afterward, since
    /// `execute` already performs a check internally.
    fn is_precompile(&self, address: H160) -> bool {
        address == *PRECOMPILE_ADDRESS
    }
}

mock! {
    #[derive(Debug)]
    pub CurrencySwap {}
    impl primitives_currency_swap::CurrencySwap<EvmAccountId, AccountId> for CurrencySwap {
        type From = EvmBalances;
        type To = Balances;
        type Error = DispatchError;

        fn swap(
            imbalance: <EvmBalances as Currency<EvmAccountId>>::NegativeImbalance,
        ) -> Result<<Balances as Currency<AccountId>>::NegativeImbalance, DispatchError>;
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
