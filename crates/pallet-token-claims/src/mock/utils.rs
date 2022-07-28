//! Mock utils.

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{Deserialize, Serialize};
use mockall::mock;
use primitives_ethereum::EcdsaSignature;
use scale_info::TypeInfo;

use super::*;
use crate::{traits, types::EthereumSignatureMessageParams};

type AccountId = <Test as frame_system::Config>::AccountId;

#[derive(
    Debug, Clone, Decode, Encode, MaxEncodedLen, TypeInfo, PartialEq, Eq, Serialize, Deserialize,
)]
pub struct MockVestingSchedule;

mock! {
    #[derive(Debug)]
    pub VestingInterface {}
    impl traits::VestingInterface for VestingInterface {
        type AccountId = AccountId;
        type Balance = crate::BalanceOf<Test>;
        type Schedule = MockVestingSchedule;

        fn lock_under_vesting(
            account: &<Self as traits::VestingInterface>::AccountId,
            balance_to_lock: <Self as traits::VestingInterface>::Balance,
            schedule: <Self as traits::VestingInterface>::Schedule,
        ) -> frame_support::dispatch::DispatchResult;
    }
}

mock! {
    #[derive(Debug)]
    pub EthereumSignatureVerifier {}

    impl traits::PreconstructedMessageVerifier for EthereumSignatureVerifier {
        type MessageParams = EthereumSignatureMessageParams<AccountId>;

        fn recover_signer(
            message_params: <Self as traits::PreconstructedMessageVerifier>::MessageParams,
            signature: EcdsaSignature,
        ) -> Option<primitives_ethereum::EthereumAddress>;
    }
}
