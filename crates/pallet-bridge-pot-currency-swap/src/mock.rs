//! The mock for the pallet.

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
use mockall::mock;
use sp_core::H256;

use crate::{self as pallet_bridge_pot_currency_swap};

pub(crate) const EXISTENTIAL_DEPOSIT: u64 = 10;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub(crate) type AccountId = u64;
type Balance = u64;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        BalancesLeft: pallet_balances::<Instance1>,
        BalancesRight: pallet_balances::<Instance2>,
        SwapBridgePot: pallet_pot::<Instance1>,
        SwapBridge: pallet_bridge_pot_currency_swap::<Instance1>,
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

type BalancesInstanceLeft = pallet_balances::Instance1;
type BalancesInstanceRight = pallet_balances::Instance2;

impl pallet_balances::Config<BalancesInstanceLeft> for Test {
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

impl pallet_balances::Config<BalancesInstanceRight> for Test {
    type Balance = u64;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<EXISTENTIAL_DEPOSIT>;
    type AccountStore = StorageMapShim<
        pallet_balances::Account<Test, BalancesInstanceRight>,
        frame_system::Provider<Test>,
        AccountId,
        pallet_balances::AccountData<Balance>,
    >;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

parameter_types! {
    pub const SwapBridgePotPalletId: PalletId = PalletId(*b"humanod1");
}

type PotInstanceSwapBridge = pallet_pot::Instance1;

impl pallet_pot::Config<PotInstanceSwapBridge> for Test {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = AccountId;
    type PalletId = SwapBridgePotPalletId;
    type Currency = BalancesLeft;
}

mock! {
    #[derive(Debug)]
    pub GenesisVerifier {}

    impl crate::GenesisVerifier for GenesisVerifier {
        fn verify() -> bool;
    }
}

parameter_types! {
    pub const SwapBridgePalletId: PalletId = PalletId(*b"hmsb/ne1");
}

parameter_types! {
    pub SwapBridgePotAccountId: AccountId = SwapBridgePot::account_id();
}

type BridgeInstanceNativeToEvmSwap = pallet_bridge_pot_currency_swap::Instance1;

impl pallet_bridge_pot_currency_swap::Config<BridgeInstanceNativeToEvmSwap> for Test {
    type AccountIdFrom = AccountId;
    type AccountIdTo = AccountId;
    type CurrencyFrom = BalancesLeft;
    type CurrencyTo = BalancesLeft;
    type BalanceConverter = Identity;
    type PotFrom = SwapBridgePotAccountId;
    type PotTo = SwapBridgePotAccountId;
    type GenesisVerifier = MockGenesisVerifier;
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
