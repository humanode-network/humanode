use super::*;

pub struct HumanodeCurrency(Balances);

impl frame_support::traits::Currency<Runtime::AccountId> for HumanodeCurrency {
    type Balance = Balances::Balance;

    type PositiveImbalance = Balances::PositiveImbalance;

    type NegativeImbalance = Balances::NegativeImbalance;

    fn total_balance(who: &Runtime::AccountId) -> Self::Balance {
        Balances::total_balance(who)
    }

    fn can_slash(who: &Runtime::AccountId, value: Self::Balance) -> bool {
        Balances::can_slash(who, value)
    }

    fn total_issuance() -> Self::Balance {
        Balances::total_issuance()
    }

    fn minimum_balance() -> Self::Balance {
        Balances::minimum_balance()
    }

    fn burn(amount: Self::Balance) -> Self::PositiveImbalance {
        panic!("no");
    }

    fn issue(amount: Self::Balance) -> Self::NegativeImbalance {
        panic!("no");
    }

    fn free_balance(who: &Runtime::AccountId) -> Self::Balance {
        Balances::free_balance(who)
    }

    fn ensure_can_withdraw(
        who: &Runtime::AccountId,
        amount: Self::Balance,
        reasons: frame_support::traits::WithdrawReasons,
        new_balance: Self::Balance,
    ) -> frame_support::dispatch::DispatchResult {
        Balances::ensure_can_withdraw(who, amount, reasons, new_balance)
    }

    fn transfer(
        source: &Runtime::AccountId,
        dest: &Runtime::AccountId,
        value: Self::Balance,
        existence_requirement: frame_support::traits::ExistenceRequirement,
    ) -> frame_support::dispatch::DispatchResult {
        Balances::transfer(source, dest, value, existence_requirement)
    }

    fn slash(
        who: &Runtime::AccountId,
        value: Self::Balance,
    ) -> (Self::NegativeImbalance, Self::Balance) {
        Balances::slash(who, value)
    }

    fn deposit_into_existing(
        who: &Runtime::AccountId,
        value: Self::Balance,
    ) -> Result<Self::PositiveImbalance, sp_runtime::DispatchError> {
        Balances::deposit_into_existing(who, value)
    }

    fn deposit_creating(who: &Runtime::AccountId, value: Self::Balance) -> Self::PositiveImbalance {
        Balances::deposit_creating(who, value)
    }

    fn withdraw(
        who: &Runtime::AccountId,
        value: Self::Balance,
        reasons: frame_support::traits::WithdrawReasons,
        liveness: frame_support::traits::ExistenceRequirement,
    ) -> Result<Self::NegativeImbalance, sp_runtime::DispatchError> {
        Balances::withdraw(who, value, reasons, liveness)
    }

    fn make_free_balance_be(
        who: &Runtime::AccountId,
        balance: Self::Balance,
    ) -> frame_support::traits::SignedImbalance<Self::Balance, Self::PositiveImbalance> {
        Balances::make_free_balance_be(who, balance)
    }
}
