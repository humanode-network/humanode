use std::collections::BTreeMap;

use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64, FindAuthor},
    weights::Weight,
};
use pallet_evm::{AddressMapping, EnsureAddressNever, FixedGasWeightMapping};
use sp_core::{H160, H256, U256};
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage, ConsensusEngineId,
};
use sp_std::{boxed::Box, prelude::*, str::FromStr};

use crate::{self as pallet_evm_balances, *};

pub(crate) const INIT_BALANCE: u64 = 10_000_000_000_000_000;

/// Alice account.
pub const ALICE: u64 = 0x5234;

/// Alice H160 account.
pub const ALICE_H160: H160 = H160(hex_literal::hex!(
    "0000000000000000000000000000000000005234"
));

/// Bob account.
pub const BOB: u64 = 0x4235;

/// Bob H160 account.
pub const BOB_H160: H160 = H160(hex_literal::hex!(
    "0000000000000000000000000000000000004235"
));

/// H160 into u64 address mapper.
pub struct H160IntoU64;

impl AddressMapping<u64> for H160IntoU64 {
    fn into_account_id(address: H160) -> u64 {
        address.to_low_u64_be()
    }
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime! {
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Timestamp: pallet_timestamp,
        EvmSystem: pallet_evm_system,
        EvmBalances: pallet_evm_balances,
        EVM: pallet_evm,
    }
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = generic::Header<u64, BlakeTwo256>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl pallet_evm_system::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = u64;
    type Index = u64;
    type AccountData = AccountData<u64>;
    type IsPrecompile = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
}

impl pallet_evm_balances::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = u64;
    type Balance = u64;
    type ExistentialDeposit = ConstU64<1>;
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

pub struct FixedGasPrice;

impl pallet_evm::FeeCalculator for FixedGasPrice {
    fn min_gas_price() -> (U256, Weight) {
        // Return some meaningful gas price and weight
        (1_000_000_000u128.into(), Weight::from_parts(7u64, 0))
    }
}

pub struct FindAuthorTruncated;

impl FindAuthor<H160> for FindAuthorTruncated {
    fn find_author<'a, I>(_digests: I) -> Option<H160>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        Some(H160::from_str("1234500000000000000000000000000000000000").unwrap())
    }
}

const BLOCK_GAS_LIMIT: u64 = 150_000_000;
const MAX_POV_SIZE: u64 = 5 * 1024 * 1024;

parameter_types! {
    pub BlockGasLimit: U256 = U256::max_value();
    pub const GasLimitPovSizeRatio: u64 = BLOCK_GAS_LIMIT.saturating_div(MAX_POV_SIZE);
    pub WeightPerGas: Weight = Weight::from_parts(20_000, 0);
}

impl pallet_evm::Config for Test {
    type AccountProvider = EvmSystem;
    type FeeCalculator = FixedGasPrice;
    type GasWeightMapping = FixedGasWeightMapping<Self>;
    type WeightPerGas = WeightPerGas;
    type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
    type CallOrigin =
        EnsureAddressNever<<Self::AccountProvider as pallet_evm::AccountProvider>::AccountId>;
    type WithdrawOrigin =
        EnsureAddressNever<<Self::AccountProvider as pallet_evm::AccountProvider>::AccountId>;
    type AddressMapping = H160IntoU64;
    type Currency = EvmBalances;
    type RuntimeEvent = RuntimeEvent;
    type PrecompilesType = ();
    type PrecompilesValue = ();
    type ChainId = ();
    type BlockGasLimit = BlockGasLimit;
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    type OnChargeTransaction = ();
    type OnCreate = ();
    type FindAuthor = FindAuthorTruncated;
    type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
    type Timestamp = Timestamp;
    type WeightInfo = ();
}

/// Build test externalities from the custom genesis.
/// Using this call requires manual assertions on the genesis init logic.
pub fn new_test_ext() -> sp_io::TestExternalities {
    // Build genesis.
    let config = GenesisConfig {
        evm: EVMConfig {
            accounts: {
                let mut map = BTreeMap::new();
                let init_genesis_account = fp_evm::GenesisAccount {
                    balance: INIT_BALANCE.into(),
                    code: Default::default(),
                    nonce: Default::default(),
                    storage: Default::default(),
                };
                map.insert(ALICE_H160, init_genesis_account.clone());
                map.insert(BOB_H160, init_genesis_account);
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
