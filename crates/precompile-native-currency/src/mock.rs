//! The mock for the precompile.

use frame_support::{
    once_cell::sync::Lazy,
    sp_io,
    sp_runtime::{
        self,
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    },
    traits::{ConstU16, ConstU32, ConstU64},
    weights::Weight,
};
use frame_system as system;
use precompile_utils::precompile_set::{PrecompileAt, PrecompileSetBuilder};
use sp_core::{ConstU128, H160, H256, U256};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub(crate) type AccountId = sp_runtime::AccountId32;
pub(crate) type EvmAccountId = H160;
pub(crate) type Balance = u128;

pub(crate) const NAME: &str = "Wrapped eHMND";
pub(crate) const SYMBOL: &str = "WeHMND";
pub(crate) const DECIMALS: u8 = 18;
pub(crate) const GAS_COST: u64 = 200;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub struct Test
    where
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
        Erc20: pallet_erc20_support,
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
    type ExistentialDeposit = ConstU128<2>;
    type AccountStore = System;
    type MaxLocks = ();
    type HoldIdentifier = ();
    type FreezeIdentifier = ();
    type MaxReserves = ();
    type MaxHolds = ConstU32<0>;
    type MaxFreezes = ConstU32<0>;
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl pallet_evm_system::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = EvmAccountId;
    type Index = u64;
    type AccountData = pallet_evm_balances::AccountData<Balance>;
    type IsPrecompile = ();
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

pub struct EvmBalancesErc20Metadata;

impl pallet_erc20_support::Metadata for EvmBalancesErc20Metadata {
    fn name() -> &'static str {
        NAME
    }

    fn symbol() -> &'static str {
        SYMBOL
    }

    fn decimals() -> u8 {
        DECIMALS
    }
}

impl pallet_erc20_support::Config for Test {
    type AccountId = EvmAccountId;
    type Currency = EvmBalances;
    type Allowance = U256;
    type Metadata = EvmBalancesErc20Metadata;
}

pub(crate) static GAS_PRICE: Lazy<U256> = Lazy::new(|| 1_000_000_000u128.into());

pub struct FixedGasPrice;
impl fp_evm::FeeCalculator for FixedGasPrice {
    fn min_gas_price() -> (U256, Weight) {
        // Return some meaningful gas price and weight
        (*GAS_PRICE, Weight::from_parts(7u64, 0))
    }
}

pub(crate) static PRECOMPILE_ADDRESS: Lazy<H160> = Lazy::new(|| H160::from_low_u64_be(0x802));

pub(crate) type EvmBalancesErc20Precompile = crate::NativeCurrency<Test, ConstU64<GAS_COST>>;

pub type Precompiles<R> =
    PrecompileSetBuilder<R, (PrecompileAt<PrecompileAddress, EvmBalancesErc20Precompile>,)>;

frame_support::parameter_types! {
    pub BlockGasLimit: U256 = U256::max_value();
    pub GasLimitPovSizeRatio: u64 = 0;
    pub WeightPerGas: Weight = Weight::from_parts(20_000, 0);
    pub PrecompileAddress: H160 = *PRECOMPILE_ADDRESS;
    pub PrecompilesValue: Precompiles<Test> = Precompiles::new();
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
    type PrecompilesType = Precompiles<Self>;
    type PrecompilesValue = PrecompilesValue;
    type ChainId = ();
    type BlockGasLimit = BlockGasLimit;
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    type OnChargeTransaction = ();
    type OnCreate = ();
    type FindAuthor = ();
    type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
    type Timestamp = Timestamp;
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
