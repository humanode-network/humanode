use std::collections::BTreeMap;

use frame_support::{
    once_cell::sync::Lazy,
    parameter_types, sp_io,
    sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, Identity, IdentityLookup},
        BuildStorage,
    },
    traits::{ConstU128, ConstU32, ConstU64},
    weights::Weight,
};
use pallet_ethereum::PostLogContent as EthereumPostLogContent;
use sp_core::{Get, H160, H256, U256};

use crate::{self as pallet_native_to_evm_swap};

pub const INIT_BALANCE: u128 = 10_000_000_000_000_000;
// Add some tokens to test swap with full balance.
pub const BRIDGE_INIT_BALANCE: u128 = INIT_BALANCE + 100;

pub fn alice() -> AccountId {
    AccountId::from(hex_literal::hex!(
        "1100000000000000000000000000000000000000000000000000000000000011"
    ))
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub type AccountId = frame_support::sp_runtime::AccountId32;
pub type EvmAccountId = H160;
pub type Balance = u128;

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
        Ethereum: pallet_ethereum,
        NativeToEvmSwap: pallet_native_to_evm_swap,
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
    type AccountId = AccountId;
    type Lookup = IdentityLookup<AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<2>; // 2 because we test the account kills via 1 balance
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

parameter_types! {
    pub const MinimumPeriod: u64 = 1000;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

pub static GAS_PRICE: Lazy<U256> = Lazy::new(|| 1_000_000_000u128.into());
pub struct FixedGasPrice;
impl fp_evm::FeeCalculator for FixedGasPrice {
    fn min_gas_price() -> (U256, Weight) {
        // Return some meaningful gas price and weight
        (*GAS_PRICE, Weight::from_parts(7u64, 0))
    }
}

parameter_types! {
    pub BlockGasLimit: U256 = U256::max_value();
    pub GasLimitPovSizeRatio: u64 = 0;
    pub WeightPerGas: Weight = Weight::from_parts(20_000, 0);
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
    type PrecompilesType = ();
    type PrecompilesValue = ();
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

parameter_types! {
    pub const PostBlockAndTxnHashes: EthereumPostLogContent = EthereumPostLogContent::BlockAndTxnHashes;
}

impl pallet_ethereum::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
    type PostLogContent = PostBlockAndTxnHashes;
    type ExtraDataLength = ConstU32<30>;
}

pub struct BridgePotNative;

impl Get<AccountId> for BridgePotNative {
    fn get() -> AccountId {
        AccountId::from(hex_literal::hex!(
            "1000000000000000000000000000000000000000000000000000000000000001"
        ))
    }
}

pub struct BridgePotEvm;

impl Get<EvmAccountId> for BridgePotEvm {
    fn get() -> EvmAccountId {
        EvmAccountId::from(hex_literal::hex!(
            "1000000000000000000000000000000000000001"
        ))
    }
}

impl pallet_native_to_evm_swap::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type EvmAccountId = EvmAccountId;
    type NativeToken = Balances;
    type EvmToken = EvmBalances;
    type BalanceConverterNativeToEvm = Identity;
    type BridgePotNative = BridgePotNative;
    type BridgePotEvm = BridgePotEvm;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    // Build genesis.
    let config = GenesisConfig {
        balances: BalancesConfig {
            balances: vec![
                (BridgePotNative::get(), BRIDGE_INIT_BALANCE),
                (alice(), INIT_BALANCE),
            ],
        },
        evm: EVMConfig {
            accounts: {
                let mut map = BTreeMap::new();
                map.insert(
                    BridgePotEvm::get(),
                    fp_evm::GenesisAccount {
                        balance: BRIDGE_INIT_BALANCE.into(),
                        code: Default::default(),
                        nonce: Default::default(),
                        storage: Default::default(),
                    },
                );
                map
            },
        },
        ..Default::default()
    };
    let storage = config.build_storage().unwrap();

    // Make test externalities from the storage.
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
