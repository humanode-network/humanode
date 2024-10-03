//! The mock for the pallet.

// Allow simple integer_arithmetic in tests.
#![allow(clippy::arithmetic_side_effects)]

use frame_support::{
    parameter_types, sp_io,
    traits::{ConstU32, ConstU64},
    PalletId,
};
use primitives_ethereum::{EcdsaSignature, EthereumAddress};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

use crate::{self as pallet_token_claims};

mod utils;
pub use self::utils::*;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub struct Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Pot: pallet_pot::{Pallet, Config<T>, Event<T>},
        TokenClaims: pallet_token_claims::{Pallet, Call, Storage, Config<T>, Event<T>},
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
    type HoldIdentifier = ();
    type FreezeIdentifier = ();
    type MaxReserves = ();
    type MaxHolds = ConstU32<0>;
    type MaxFreezes = ConstU32<0>;
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

parameter_types! {
    pub const PotPalletId: PalletId = PalletId(*b"tokenclm");
}

impl pallet_pot::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AccountId = u64;
    type Currency = Balances;
    type PalletId = PotPalletId;
}

parameter_types! {
    pub PotAccountId: u64 = Pot::account_id();
}

impl pallet_token_claims::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type PotAccountId = PotAccountId;
    type VestingSchedule = MockVestingSchedule;
    type VestingInterface = MockVestingInterface;
    type EthereumSignatureVerifier = MockEthereumSignatureVerifier;
    type WeightInfo = ();
}

pub const TREASURY: u64 = 1001;

pub enum EthAddr {
    Existing,
    SecondExisting,
    New,
    Unknown,
    Other(u8),
}

impl From<EthAddr> for u8 {
    fn from(eth_addr: EthAddr) -> Self {
        match eth_addr {
            EthAddr::Existing => 1,
            EthAddr::SecondExisting => 2,
            EthAddr::New => 3,
            EthAddr::Unknown => 0xff,
            EthAddr::Other(val) => val,
        }
    }
}

/// Utility function for creating dummy ethereum accounts.
pub fn eth(val: EthAddr) -> EthereumAddress {
    let mut addr = [0; 20];
    addr[19] = val.into();
    EthereumAddress(addr)
}

/// Utility function for creating dummy ecdsa signatures.
pub fn sig(num: u8) -> EcdsaSignature {
    let mut signature = [0; 65];
    signature[64] = num;
    EcdsaSignature(signature)
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let genesis_config = GenesisConfig {
        system: Default::default(),
        balances: BalancesConfig {
            balances: vec![
                (
                    Pot::account_id(),
                    30 /* tokens sum */ +
                1, /* existential deposit */
                ),
                (TREASURY, 1000),
            ],
        },
        pot: Default::default(),
        token_claims: TokenClaimsConfig {
            claims: [
                (eth(EthAddr::Existing), 10, MockVestingSchedule),
                (eth(EthAddr::SecondExisting), 20, MockVestingSchedule),
            ]
            .into_iter()
            .map(|(eth_address, balance, vesting)| {
                (
                    eth_address,
                    pallet_token_claims::types::ClaimInfo { balance, vesting },
                )
            })
            .collect(),
            total_claimable: Some(30),
        },
    };
    new_test_ext_with(genesis_config)
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext_with(genesis_config: GenesisConfig) -> sp_io::TestExternalities {
    let storage = genesis_config.build_storage().unwrap();
    storage.into()
}
