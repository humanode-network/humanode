use fp_evm::{Context, ExitError, ExitReason, PrecompileHandle, Transfer};
use frame_support::traits::{ConstU16, ConstU64};
use frame_system as system;
use mockall::predicate::*;
use mockall::*;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
    BuildStorage, MultiSignature,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

/// An index to a block.
pub type BlockNumber = u64;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        EvmAccountsMapping: pallet_evm_accounts_mapping,
    }
);

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

pub struct MockVerifier;

impl pallet_evm_accounts_mapping::SignedClaimVerifier for MockVerifier {
    type AccountId = AccountId;
    fn verify(
        _account_id: AccountId,
        _signature: pallet_evm_accounts_mapping::Secp256k1EcdsaSignature,
    ) -> Option<pallet_evm_accounts_mapping::EvmAddress> {
        panic!("should be unused in tests")
    }
}

impl pallet_evm_accounts_mapping::Config for Test {
    type Event = Event;
    type Verifier = MockVerifier;
}

mock! {
    pub PrecompileHandle {}

    impl PrecompileHandle for PrecompileHandle {
        fn call(
            &mut self,
            to: primitive_types::H160,
            transfer: Option<Transfer>,
            input: Vec<u8>,
            gas_limit: Option<u64>,
            is_static: bool,
            context: &Context,
        ) -> (ExitReason, Vec<u8>);

        fn record_cost(&mut self, cost: u64) -> Result<(), ExitError>;

        fn remaining_gas(&self) -> u64;

        fn log(&mut self, address: primitive_types::H160, topics: Vec<primitive_types::H256>, data: Vec<u8>) -> Result<(), ExitError>;

        fn code_address(&self) -> primitive_types::H160;

        fn input(&self) -> &[u8];

        fn context(&self) -> &Context;

        fn is_static(&self) -> bool;

        fn gas_limit(&self) -> Option<u64>;
    }
}

/// Build test externalities from the default genesis.
pub fn new_test_ext() -> sp_io::TestExternalities {
    // Build genesis.
    let config = GenesisConfig {
        ..Default::default()
    };
    let storage = config.build_storage().unwrap();

    // Make test externalities from the storage.
    storage.into()
}
