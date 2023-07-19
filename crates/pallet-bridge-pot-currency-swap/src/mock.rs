//! The mock for the pallet.

// Allow simple integer arithmetic in tests.
#![allow(clippy::integer_arithmetic)]

use frame_support::{
    parameter_types,
    sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, Identity, IdentityLookup},
    },
    traits::{ConstU32, ConstU64},
    PalletId,
};
use mockall::mock;
use sp_core::{H160, H256};

use crate::{self as pallet_bridge_pot_currency_swap};

pub(crate) const EXISTENTIAL_DEPOSIT: u64 = 10;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub(crate) type AccountId = u64;
pub(crate) type EvmAccountId = H160;
type Balance = u64;

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
        NativeToEvmSwapBridgePot: pallet_pot::<Instance1>,
        EvmToNativeSwapBridgePot: pallet_pot::<Instance2>,
        NativeToEvmSwapBridge: pallet_bridge_pot_currency_swap::<Instance1>,
        EvmToNativeSwapBridge: pallet_bridge_pot_currency_swap::<Instance2>,
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
    type Balance = u64;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<EXISTENTIAL_DEPOSIT>;
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
    type ExistentialDeposit = ConstU64<EXISTENTIAL_DEPOSIT>;
    type AccountStore = EvmSystem;
    type DustRemoval = ();
}

parameter_types! {
    pub const NativeToEvmSwapBridgePotPalletId: PalletId = PalletId(*b"hmcs/ne1");
    pub const EvmToNativeSwapBridgePotPalletId: PalletId = PalletId(*b"hmcs/en1");
}

type PotInstanceNativeToEvmSwapBridge = pallet_pot::Instance1;
type PotInstanceEvmToNativeSwapBridge = pallet_pot::Instance2;

impl pallet_pot::Config<PotInstanceNativeToEvmSwapBridge> for Test {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = AccountId;
    type PalletId = NativeToEvmSwapBridgePotPalletId;
    type Currency = Balances;
}

impl pallet_pot::Config<PotInstanceEvmToNativeSwapBridge> for Test {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = EvmAccountId;
    type PalletId = EvmToNativeSwapBridgePotPalletId;
    type Currency = EvmBalances;
}

mock! {
    #[derive(Debug)]
    pub GenesisVerifier {}

    impl crate::GenesisVerifier for GenesisVerifier {
        fn verify() -> bool;
    }
}

parameter_types! {
    pub const NativeToEvmSwapBridgePalletId: PalletId = PalletId(*b"hmsb/ne1");
    pub const EvmToNativeSwapBridgePalletId: PalletId = PalletId(*b"hmsb/en1");
}

parameter_types! {
    pub NativeToEvmSwapBridgePotAccountId: AccountId = NativeToEvmSwapBridgePot::account_id();
    pub EvmToNativeSwapBridgePotAccountId: EvmAccountId = EvmToNativeSwapBridgePot::account_id();
}

type BridgeInstanceNativeToEvmSwap = pallet_bridge_pot_currency_swap::Instance1;
type BridgeInstanceEvmToNativeSwap = pallet_bridge_pot_currency_swap::Instance2;

impl pallet_bridge_pot_currency_swap::Config<BridgeInstanceNativeToEvmSwap> for Test {
    type AccountIdFrom = AccountId;
    type AccountIdTo = EvmAccountId;
    type CurrencyFrom = Balances;
    type CurrencyTo = EvmBalances;
    type BalanceConverter = Identity;
    type PotFrom = NativeToEvmSwapBridgePotAccountId;
    type PotTo = EvmToNativeSwapBridgePotAccountId;
    type GenesisVerifier = MockGenesisVerifier;
}

impl pallet_bridge_pot_currency_swap::Config<BridgeInstanceEvmToNativeSwap> for Test {
    type AccountIdFrom = EvmAccountId;
    type AccountIdTo = AccountId;
    type CurrencyFrom = EvmBalances;
    type CurrencyTo = Balances;
    type BalanceConverter = Identity;
    type PotFrom = EvmToNativeSwapBridgePotAccountId;
    type PotTo = NativeToEvmSwapBridgePotAccountId;
    type GenesisVerifier = MockGenesisVerifier;
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
