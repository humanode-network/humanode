use frame_support::traits::fungible::Inspect;
use frame_support::traits::Currency;

use super::*;

pub struct HumanodeCurrency(Balances);

impl Currency<AccountId> for HumanodeCurrency {
    type Balance = <Balances as Currency<AccountId>>::Balance;

    type PositiveImbalance = <Balances as Currency<AccountId>>::PositiveImbalance;

    type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

    fn total_balance(who: &AccountId) -> Self::Balance {
        <Balances as Currency<AccountId>>::total_balance(who)
    }

    fn can_slash(who: &AccountId, value: Self::Balance) -> bool {
        <Balances as Currency<AccountId>>::can_slash(who, value)
    }

    fn total_issuance() -> Self::Balance {
        <Balances as Currency<AccountId>>::total_issuance()
    }

    fn minimum_balance() -> Self::Balance {
        <Balances as Currency<AccountId>>::minimum_balance()
    }

    fn burn(_amount: Self::Balance) -> Self::PositiveImbalance {
        panic!("no");
    }

    fn issue(_amount: Self::Balance) -> Self::NegativeImbalance {
        panic!("no");
    }

    fn free_balance(who: &AccountId) -> Self::Balance {
        <Balances as Currency<AccountId>>::free_balance(who)
    }

    fn ensure_can_withdraw(
        who: &AccountId,
        amount: Self::Balance,
        reasons: frame_support::traits::WithdrawReasons,
        new_balance: Self::Balance,
    ) -> frame_support::dispatch::DispatchResult {
        <Balances as Currency<AccountId>>::ensure_can_withdraw(who, amount, reasons, new_balance)
    }

    fn transfer(
        source: &AccountId,
        dest: &AccountId,
        value: Self::Balance,
        existence_requirement: frame_support::traits::ExistenceRequirement,
    ) -> frame_support::dispatch::DispatchResult {
        <Balances as Currency<AccountId>>::transfer(source, dest, value, existence_requirement)
    }

    fn slash(who: &AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
        <Balances as Currency<AccountId>>::slash(who, value)
    }

    fn deposit_into_existing(
        who: &AccountId,
        value: Self::Balance,
    ) -> Result<Self::PositiveImbalance, sp_runtime::DispatchError> {
        <Balances as Currency<AccountId>>::deposit_into_existing(who, value)
    }

    fn deposit_creating(who: &AccountId, value: Self::Balance) -> Self::PositiveImbalance {
        <Balances as Currency<AccountId>>::deposit_creating(who, value)
    }

    fn withdraw(
        who: &AccountId,
        value: Self::Balance,
        reasons: frame_support::traits::WithdrawReasons,
        liveness: frame_support::traits::ExistenceRequirement,
    ) -> Result<Self::NegativeImbalance, sp_runtime::DispatchError> {
        <Balances as Currency<AccountId>>::withdraw(who, value, reasons, liveness)
    }

    fn make_free_balance_be(
        who: &AccountId,
        balance: Self::Balance,
    ) -> frame_support::traits::SignedImbalance<Self::Balance, Self::PositiveImbalance> {
        <Balances as Currency<AccountId>>::make_free_balance_be(who, balance)
    }
}

impl Inspect<AccountId> for HumanodeCurrency {
    type Balance = <Balances as Inspect<AccountId>>::Balance;

    fn total_issuance() -> Self::Balance {
        <Balances as Inspect<AccountId>>::total_issuance()
    }

    fn minimum_balance() -> Self::Balance {
        <Balances as Inspect<AccountId>>::minimum_balance()
    }

    fn balance(who: &AccountId) -> Self::Balance {
        <Balances as Inspect<AccountId>>::balance(who)
    }

    fn reducible_balance(who: &AccountId, keep_alive: bool) -> Self::Balance {
        <Balances as Inspect<AccountId>>::reducible_balance(who, keep_alive)
    }

    fn can_deposit(
        who: &AccountId,
        amount: Self::Balance,
        mint: bool,
    ) -> frame_support::traits::tokens::DepositConsequence {
        <Balances as Inspect<AccountId>>::can_deposit(who, amount, mint)
    }

    fn can_withdraw(
        who: &AccountId,
        amount: Self::Balance,
    ) -> frame_support::traits::tokens::WithdrawConsequence<Self::Balance> {
        <Balances as Inspect<AccountId>>::can_withdraw(who, amount)
    }
}
