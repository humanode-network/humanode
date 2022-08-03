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
