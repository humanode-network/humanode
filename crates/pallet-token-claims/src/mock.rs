//! The mock for the pallet.

use frame_support::{
    ord_parameter_types, parameter_types,
    traits::{ConstU32, ConstU64},
};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

use crate::{self as pallet_token_claims};

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

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> frame_support::sp_io::TestExternalities {
    let genesis_config = GenesisConfig {
        system: Default::default(),
        balances: Default::default(),
        token_claims: Default::default(),
    };
    use sp_runtime::BuildStorage;
    let storage = genesis_config.build_storage().unwrap();
    storage.into()
}

/// Test mocks
mod mocks {
    use codec::{Decode, Encode, MaxEncodedLen};
    use frame_support::{Deserialize, Serialize};
    use scale_info::TypeInfo;

    use super::*;
    use crate::{
        traits::{self, PreconstructedMessageVerifier},
        types::EthereumSignatureMessageParams,
    };

    #[derive(
        Debug, Clone, Decode, Encode, MaxEncodedLen, TypeInfo, PartialEq, Eq, Serialize, Deserialize,
    )]
    pub struct MockVestingSchedule;

    #[derive(Debug)]
    pub struct MockVestingInterface;

    impl traits::VestingInterface for MockVestingInterface {
        type AccountId = <Test as frame_system::Config>::AccountId;
        type Balance = crate::BalanceOf<Test>;
        type Schedule = MockVestingSchedule;

        fn lock_under_vesting(
            _account: &Self::AccountId,
            _balance_to_lock: Self::Balance,
            _schedule: Self::Schedule,
        ) -> frame_support::dispatch::DispatchResult {
            Ok(())
        }
    }

    #[derive(Debug)]
    pub struct MockEthereumSignatureVerifier;

    impl PreconstructedMessageVerifier for MockEthereumSignatureVerifier {
        type MessageParams =
            EthereumSignatureMessageParams<<Test as frame_system::Config>::AccountId>;

        fn recover_signer(
            _message_params: Self::MessageParams,
            _signature: primitives_ethereum::EcdsaSignature,
        ) -> Option<primitives_ethereum::EthereumAddress> {
            None
        }
    }
}
use self::mocks::*;
