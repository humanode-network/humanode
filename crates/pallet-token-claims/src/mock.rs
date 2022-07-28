//! The mock for the pallet.

use frame_support::{
    ord_parameter_types, parameter_types, sp_io,
    traits::{ConstU32, ConstU64},
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
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        TokenClaims: pallet_token_claims::{Pallet, Call, Storage, Config<T>, Event<T>},
    }
);

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<u64>;
    type Header = Header;
    type Event = Event;
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
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<1>;
    type AccountStore = System;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

parameter_types! {
    pub Prefix: &'static [u8] = b"Pay RUSTs to the TEST account:";
}
ord_parameter_types! {
    pub const Six: u64 = 6;
}

impl pallet_token_claims::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type VestingSchedule = MockVestingSchedule;
    type VestingInterface = MockVestingInterface;
    type EthereumSignatureVerifier = MockEthereumSignatureVerifier;
    type WeightInfo = ();
}

/// Utility function for creating dummy ethereum accounts.
pub fn eth(num: u8) -> EthereumAddress {
    let mut addr = [0; 20];
    addr[19] = num;
    EthereumAddress(addr)
}

pub fn sig(num: u8) -> EcdsaSignature {
    let mut signature = [0; 65];
    signature[64] = num;
    EcdsaSignature(signature)
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let genesis_config = GenesisConfig {
        system: Default::default(),
        balances: Default::default(),
        token_claims: TokenClaimsConfig {
            claims: [(eth(1), 10, None), (eth(2), 20, Some(MockVestingSchedule))]
                .into_iter()
                .map(|(eth_address, balance, vesting)| {
                    (
                        eth_address,
                        pallet_token_claims::types::ClaimInfo { balance, vesting },
                    )
                })
                .collect(),
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
