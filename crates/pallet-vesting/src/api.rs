//! The runtime APIs.

use codec::{Decode, Encode};
use frame_support::sp_runtime::DispatchError;
use scale_info::TypeInfo;

/// An error that can occur while evaluating the lock logic.
#[derive(Debug, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub enum EvaluationError {
    /// No vesting is found for the given account.
    NoVesting,
    /// Something went wrong during the computation.
    Computation(DispatchError),
}

sp_api::decl_runtime_apis! {
    /// A runtime API for evaluating the locking logic.
    pub trait VestingEvaluationApi<AccountId, Balance>
    where
        AccountId: Encode,
        Balance: Decode,
    {
        /// Compute the balance under lock.
        fn evaluate_lock(account: &AccountId) -> Result<Balance, EvaluationError>;
    }
}
