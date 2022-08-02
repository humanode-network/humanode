use frame_support::traits::{LockableCurrency, WithdrawReasons};

use super::*;

pub enum TokenClaimsInterface {}

impl pallet_token_claims::traits::VestingInterface for TokenClaimsInterface {
    type AccountId = AccountId;
    type Balance = Balance;
    type Schedule = ();

    fn lock_under_vesting(
        account: &Self::AccountId,
        balance_to_lock: Self::Balance,
        _schedule: Self::Schedule,
    ) -> frame_support::pallet_prelude::DispatchResult {
        <Balances as LockableCurrency<AccountId>>::set_lock(
            *b"hmnd/tc1",
            account,
            balance_to_lock,
            WithdrawReasons::RESERVE,
        );
        Ok(())
    }
}
