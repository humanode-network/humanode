//! The v2 mock that includes bridges initialization logic at runtime.

// Allow simple integer arithmetic in tests.
#![allow(clippy::integer_arithmetic)]

use frame_support::{
    parameter_types, sp_io,
    sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, Identity, IdentityLookup},
        BuildStorage,
    },
    traits::{ConstU32, ConstU64, StorageMapShim},
    PalletId,
};
use sp_core::H256;

use crate::{self as pallet_bridges_initializer_currency_swap};

pub(crate) const EXISTENTIAL_DEPOSIT_NATIVE: u64 = 10;
pub(crate) const EXISTENTIAL_DEPOSIT_EVM: u64 = 20;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub(crate) type AccountId = u64;
pub(crate) type EvmAccountId = u64;
type Balance = u64;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances::<Instance1>,
        EvmBalances: pallet_balances::<Instance2>,
        SwapBridgeNativeToEvmPot: pallet_pot::<Instance1>,
        SwapBridgeEvmToNativePot: pallet_pot::<Instance2>,
        EvmNativeBridgesInitializer: pallet_bridges_initializer_currency_swap,
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

type BalancesInstanceNative = pallet_balances::Instance1;
type BalancesInstanceEvm = pallet_balances::Instance2;

impl pallet_balances::Config<BalancesInstanceNative> for Test {
    type Balance = u64;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<EXISTENTIAL_DEPOSIT_NATIVE>;
    type AccountStore = System;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl pallet_balances::Config<BalancesInstanceEvm> for Test {
    type Balance = u64;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<EXISTENTIAL_DEPOSIT_EVM>;
    type AccountStore = StorageMapShim<
        pallet_balances::Account<Test, BalancesInstanceEvm>,
        frame_system::Provider<Test>,
        EvmAccountId,
        pallet_balances::AccountData<Balance>,
    >;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

parameter_types! {
    pub const SwapBridgeNativeToEvmPotPalletId: PalletId = PalletId(*b"humanoNE");
    pub const SwapBridgeEvmToNativePotPalletId: PalletId = PalletId(*b"humanoEN");
}

type PotInstanceSwapBridgeNativeToEvm = pallet_pot::Instance1;
type PotInstanceSwapBridgeEvmToNative = pallet_pot::Instance2;

impl pallet_pot::Config<PotInstanceSwapBridgeNativeToEvm> for Test {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = AccountId;
    type PalletId = SwapBridgeNativeToEvmPotPalletId;
    type Currency = Balances;
}

impl pallet_pot::Config<PotInstanceSwapBridgeEvmToNative> for Test {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = EvmAccountId;
    type PalletId = SwapBridgeEvmToNativePotPalletId;
    type Currency = EvmBalances;
}

parameter_types! {
    pub const SwapBridgeNativeToEvmPalletId: PalletId = PalletId(*b"hmsb/ne1");
    pub const SwapBridgeEvmToNativePalletId: PalletId = PalletId(*b"hmsb/en1");
}

parameter_types! {
    pub SwapBridgeNativeToEvmPotAccountId: AccountId = SwapBridgeNativeToEvmPot::account_id();
    pub SwapBridgeEvmToNativePotAccountId: AccountId = SwapBridgeEvmToNativePot::account_id();
}

parameter_types! {
    pub NativeTreasury: AccountId = 4200;
}

impl pallet_bridges_initializer_currency_swap::Config for Test {
    type EvmAccountId = EvmAccountId;
    type NativeCurrency = Balances;
    type EvmCurrency = EvmBalances;
    type BalanceConverterEvmToNative = Identity;
    type BalanceConverterNativeToEvm = Identity;
    type NativeEvmBridgePot = SwapBridgeNativeToEvmPotAccountId;
    type NativeTreasuryPot = NativeTreasury;
    type EvmNativeBridgePot = SwapBridgeEvmToNativePotAccountId;
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

pub fn with_runtime_lock<R>(f: impl FnOnce() -> R) -> R {
    let lock = runtime_lock();
    let res = f();
    drop(lock);
    res
}
