use frame_support::traits::fungible::Inspect;
use frame_support::traits::{Currency, Imbalance, SameOrOther, SignedImbalance, TryDrop};

use super::*;

pub struct FixedSupplyCurrency(Balances);

#[derive(Default)]
pub struct FixedSupplyPositiveImbalance(
    Option<<Balances as Currency<AccountId>>::PositiveImbalance>,
);

#[derive(Default)]
pub struct FixedSupplyNegativeImbalance(
    Option<<Balances as Currency<AccountId>>::NegativeImbalance>,
);

impl FixedSupplyPositiveImbalance {
    fn new(val: <Balances as Currency<AccountId>>::PositiveImbalance) -> Self {
        Self(Some(val))
    }

    fn must_take(&mut self) -> <Balances as Currency<AccountId>>::PositiveImbalance {
        self.0.take().unwrap()
    }

    fn must_ref(&self) -> &<Balances as Currency<AccountId>>::PositiveImbalance {
        self.0.as_ref().unwrap()
    }
}

impl FixedSupplyNegativeImbalance {
    fn new(val: <Balances as Currency<AccountId>>::NegativeImbalance) -> Self {
        Self(Some(val))
    }

    fn must_take(&mut self) -> <Balances as Currency<AccountId>>::NegativeImbalance {
        self.0.take().unwrap()
    }

    fn must_ref(&self) -> &<Balances as Currency<AccountId>>::NegativeImbalance {
        self.0.as_ref().unwrap()
    }

    fn must_mut(&mut self) -> &mut <Balances as Currency<AccountId>>::NegativeImbalance {
        self.0.as_mut().unwrap()
    }
}

impl Currency<AccountId> for FixedSupplyCurrency {
    type Balance = <Balances as Currency<AccountId>>::Balance;

    type PositiveImbalance = FixedSupplyPositiveImbalance;

    type NegativeImbalance = FixedSupplyNegativeImbalance;

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
        let (imbalance, amount) = <Balances as Currency<AccountId>>::slash(who, value);
        (FixedSupplyNegativeImbalance::new(imbalance), amount)
    }

    fn deposit_into_existing(
        who: &AccountId,
        value: Self::Balance,
    ) -> Result<Self::PositiveImbalance, sp_runtime::DispatchError> {
        <Balances as Currency<AccountId>>::deposit_into_existing(who, value)
            .map(FixedSupplyPositiveImbalance::new)
    }

    fn deposit_creating(who: &AccountId, value: Self::Balance) -> Self::PositiveImbalance {
        FixedSupplyPositiveImbalance::new(<Balances as Currency<AccountId>>::deposit_creating(
            who, value,
        ))
    }

    fn withdraw(
        who: &AccountId,
        value: Self::Balance,
        reasons: frame_support::traits::WithdrawReasons,
        liveness: frame_support::traits::ExistenceRequirement,
    ) -> Result<Self::NegativeImbalance, sp_runtime::DispatchError> {
        <Balances as Currency<AccountId>>::withdraw(who, value, reasons, liveness)
            .map(FixedSupplyNegativeImbalance::new)
    }

    fn make_free_balance_be(
        who: &AccountId,
        balance: Self::Balance,
    ) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
        match <Balances as Currency<AccountId>>::make_free_balance_be(who, balance) {
            SignedImbalance::Positive(val) => {
                SignedImbalance::Positive(FixedSupplyPositiveImbalance::new(val))
            }
            SignedImbalance::Negative(val) => {
                SignedImbalance::Negative(FixedSupplyNegativeImbalance::new(val))
            }
        }
    }
}

impl Inspect<AccountId> for FixedSupplyCurrency {
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

impl Imbalance<Balance> for FixedSupplyPositiveImbalance {
    type Opposite = FixedSupplyNegativeImbalance;

    fn zero() -> Self {
        Self::new(<Balances as Currency<AccountId>>::PositiveImbalance::zero())
    }

    fn drop_zero(mut self) -> Result<(), Self> {
        self.must_take().drop_zero().map_err(Self::new)
    }

    fn split(mut self, amount: Balance) -> (Self, Self) {
        let (left, right) = self.must_take().split(amount);
        (Self::new(left), Self::new(right))
    }

    fn merge(mut self, mut other: Self) -> Self {
        Self::new(self.must_take().merge(other.must_take()))
    }

    fn subsume(&mut self, mut other: Self) {
        self.must_take().subsume(other.must_take())
    }

    fn offset(mut self, mut other: Self::Opposite) -> SameOrOther<Self, Self::Opposite> {
        match self.must_take().offset(other.must_take()) {
            SameOrOther::None => SameOrOther::None,
            SameOrOther::Same(val) => SameOrOther::Same(Self::new(val)),
            SameOrOther::Other(val) => SameOrOther::Other(FixedSupplyNegativeImbalance::new(val)),
        }
    }

    fn peek(&self) -> Balance {
        self.must_ref().peek()
    }
}

impl Imbalance<Balance> for FixedSupplyNegativeImbalance {
    type Opposite = FixedSupplyPositiveImbalance;

    fn zero() -> Self {
        Self::new(<Balances as Currency<AccountId>>::NegativeImbalance::zero())
    }

    fn drop_zero(mut self) -> Result<(), Self> {
        self.must_take().drop_zero().map_err(Self::new)
    }

    fn split(mut self, amount: Balance) -> (Self, Self) {
        let (left, right) = self.must_take().split(amount);
        (Self::new(left), Self::new(right))
    }

    fn merge(mut self, mut other: Self) -> Self {
        Self::new(self.must_take().merge(other.must_take()))
    }

    fn subsume(&mut self, mut other: Self) {
        self.must_mut().subsume(other.must_take())
    }

    fn offset(mut self, mut other: Self::Opposite) -> SameOrOther<Self, Self::Opposite> {
        match self.must_take().offset(other.must_take()) {
            SameOrOther::None => SameOrOther::None,
            SameOrOther::Same(val) => SameOrOther::Same(Self::new(val)),
            SameOrOther::Other(val) => SameOrOther::Other(FixedSupplyPositiveImbalance::new(val)),
        }
    }

    fn peek(&self) -> Balance {
        self.must_ref().peek()
    }
}

impl TryDrop for FixedSupplyPositiveImbalance {
    fn try_drop(mut self) -> Result<(), Self> {
        self.must_take().try_drop().map_err(Self::new)
    }
}

impl TryDrop for FixedSupplyNegativeImbalance {
    fn try_drop(mut self) -> Result<(), Self> {
        self.must_take().try_drop().map_err(Self::new)
    }
}

impl Drop for FixedSupplyPositiveImbalance {
    fn drop(&mut self) {
        let val = match &self.0 {
            None => return,
            Some(val) => val,
        };

        if val != &<Balances as Currency<AccountId>>::PositiveImbalance::zero() {
            panic!("dropping a non-zero positive imbalance")
        }
    }
}

impl Drop for FixedSupplyNegativeImbalance {
    fn drop(&mut self) {
        let val = match &self.0 {
            None => return,
            Some(val) => val,
        };

        if val != &<Balances as Currency<AccountId>>::NegativeImbalance::zero() {
            panic!("dropping a non-zero negative imbalance")
        }
    }
}
