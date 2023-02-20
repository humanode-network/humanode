//! The mock for the pallet.

// Allow simple integer_arithmetic in tests.
#![allow(clippy::integer_arithmetic)]

use frame_support::{
    sp_io,
    traits::{ConstU32, ConstU64},
};
use mockall::mock;
use primitives_ethereum::{EcdsaSignature, EthereumAddress};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

use crate::{self as pallet_evm_accounts_mapping};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

mock! {
    #[derive(Debug)]
    pub SignedClaimVerifier {}

    impl pallet_evm_accounts_mapping::SignedClaimVerifier for SignedClaimVerifier {
        type AccountId = u64;

        fn verify(account_id: &<Self as pallet_evm_accounts_mapping::SignedClaimVerifier>::AccountId, signature: &EcdsaSignature) -> Option<EthereumAddress>;
    }
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

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        EvmAccountsMapping: pallet_evm_accounts_mapping::{Pallet, Call, Storage, Config<T>, Event<T>},
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
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl pallet_evm_accounts_mapping::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Verifier = MockSignedClaimVerifier;
    type WeightInfo = ();
}

pub enum EthAddr {
    Existing,
    New,
    Unknown,
}

impl From<EthAddr> for u8 {
    fn from(eth_addr: EthAddr) -> Self {
        match eth_addr {
            EthAddr::Existing => 1,
            EthAddr::New => 2,
            EthAddr::Unknown => 0xff,
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
        evm_accounts_mapping: EvmAccountsMappingConfig {
            mappings: vec![(42, eth(EthAddr::Existing))],
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
