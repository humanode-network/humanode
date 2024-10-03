//! The v2 mock.

// Allow simple integer arithmetic in tests.
#![allow(clippy::arithmetic_side_effects)]

use frame_support::{
    parameter_types,
    sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, Identity, IdentityLookup},
    },
    traits::{ConstU32, ConstU64, StorageMapShim},
    PalletId,
};
use sp_core::{ConstU16, H256};

pub(crate) const FORCE_REBALANCE_ASK_COUNTER: u16 = 1;

use super::*;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub struct Test
    where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system = 0,
        Balances: pallet_balances::<Instance1>,
        EvmBalances: pallet_balances::<Instance2>,
        SwapBridgeNativeToEvmPot: pallet_pot::<Instance1>,
        SwapBridgeEvmToNativePot: pallet_pot::<Instance2>,
        EvmNativeBridgesInitializer: pallet_balanced_currency_swap_bridges_initializer,
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

impl pallet_balances::Config<BalancesInstanceNative> for Test {
    type Balance = u64;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<EXISTENTIAL_DEPOSIT_NATIVE>;
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

impl pallet_balances::Config<BalancesInstanceEvm> for Test {
    type Balance = u64;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<EXISTENTIAL_DEPOSIT_EVM_NEW>;
    type AccountStore = StorageMapShim<
        pallet_balances::Account<Test, BalancesInstanceEvm>,
        EvmAccountId,
        pallet_balances::AccountData<Balance>,
    >;
    type MaxLocks = ();
    type HoldIdentifier = ();
    type FreezeIdentifier = ();
    type MaxReserves = ();
    type MaxHolds = ConstU32<0>;
    type MaxFreezes = ConstU32<0>;
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

impl pallet_balanced_currency_swap_bridges_initializer::Config for Test {
    type EvmAccountId = EvmAccountId;
    type NativeCurrency = Balances;
    type EvmCurrency = EvmBalances;
    type BalanceConverterEvmToNative = Identity;
    type BalanceConverterNativeToEvm = Identity;
    type NativeEvmBridgePot = SwapBridgeNativeToEvmPotAccountId;
    type NativeTreasuryPot = NativeTreasury;
    type EvmNativeBridgePot = SwapBridgeEvmToNativePotAccountId;
    type ForceRebalanceAskCounter = ConstU16<FORCE_REBALANCE_ASK_COUNTER>;
    type WeightInfo = ();
}
