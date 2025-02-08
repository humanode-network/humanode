//! Currency trait implementation.

use frame_support::traits::Currency;

use super::*;

impl<T: Config<I>, I: 'static> Currency<<T as Config<I>>::AccountId> for Pallet<T, I>
where
    T::Balance: MaybeSerializeDeserialize + Debug,
{
    type Balance = T::Balance;
    type PositiveImbalance = PositiveImbalance<T, I>;
    type NegativeImbalance = NegativeImbalance<T, I>;

    fn total_balance(who: &<T as Config<I>>::AccountId) -> Self::Balance {
        Self::account(who).total()
    }

    fn can_slash(who: &<T as Config<I>>::AccountId, value: Self::Balance) -> bool {
        if value.is_zero() {
            return true;
        }
        Self::free_balance(who) >= value
    }

    fn total_issuance() -> Self::Balance {
        TotalIssuance::<T, I>::get()
    }

    fn active_issuance() -> Self::Balance {
        <Self as fungible::Inspect<_>>::active_issuance()
    }

    fn deactivate(amount: Self::Balance) {
        <Self as fungible::Unbalanced<_>>::deactivate(amount);
    }

    fn reactivate(amount: Self::Balance) {
        <Self as fungible::Unbalanced<_>>::reactivate(amount);
    }

    fn minimum_balance() -> Self::Balance {
        T::ExistentialDeposit::get()
    }

    // Burn funds from the total issuance, returning a positive imbalance for the amount burned.
    // Is a no-op if amount to be burned is zero.
    fn burn(mut amount: Self::Balance) -> Self::PositiveImbalance {
        if amount.is_zero() {
            return PositiveImbalance::zero();
        }
        <TotalIssuance<T, I>>::mutate(|issued| {
            *issued = issued.checked_sub(&amount).unwrap_or_else(|| {
                amount = *issued;
                Zero::zero()
            });
        });
        PositiveImbalance::new(amount)
    }

    // Create new funds into the total issuance, returning a negative imbalance
    // for the amount issued.
    // Is a no-op if amount to be issued it zero.
    fn issue(mut amount: Self::Balance) -> Self::NegativeImbalance {
        if amount.is_zero() {
            return NegativeImbalance::zero();
        }
        <TotalIssuance<T, I>>::mutate(|issued| {
            *issued = issued.checked_add(&amount).unwrap_or_else(|| {
                amount = Self::Balance::max_value() - *issued;
                Self::Balance::max_value()
            })
        });
        NegativeImbalance::new(amount)
    }

    fn free_balance(who: &<T as Config<I>>::AccountId) -> Self::Balance {
        Self::account(who).free
    }

    // We don't have any existing withdrawal restrictions like locked and reserved balance.
    // Is a no-op if amount to be withdrawn is zero.
    fn ensure_can_withdraw(
        _who: &<T as Config<I>>::AccountId,
        _amount: T::Balance,
        _reasons: WithdrawReasons,
        _new_balance: T::Balance,
    ) -> DispatchResult {
        Ok(())
    }

    // Transfer some free balance from `transactor` to `dest`, respecting existence requirements.
    // Is a no-op if value to be transferred is zero or the `transactor` is the same as `dest`.
    fn transfer(
        transactor: &<T as Config<I>>::AccountId,
        dest: &<T as Config<I>>::AccountId,
        value: Self::Balance,
        existence_requirement: ExistenceRequirement,
    ) -> DispatchResult {
        if value.is_zero() || transactor == dest {
            return Ok(());
        }
        let keep_alive = match existence_requirement {
            ExistenceRequirement::KeepAlive => Preservation::Preserve,
            ExistenceRequirement::AllowDeath => Preservation::Expendable,
        };
        <Self as fungible::Mutate<_>>::transfer(transactor, dest, value, keep_alive)?;
        Ok(())
    }

    /// Slash a target account `who`, returning the negative imbalance created and any left over
    /// amount that could not be slashed.
    ///
    /// Is a no-op if `value` to be slashed is zero or the account does not exist.
    ///
    /// NOTE: `slash()` prefers free balance, but assumes that reserve balance can be drawn
    /// from in extreme circumstances. `can_slash()` should be used prior to `slash()` to avoid
    /// having to draw from reserved funds, however we err on the side of punishment if things are
    /// inconsistent or `can_slash` wasn't used appropriately.
    fn slash(
        who: &<T as Config<I>>::AccountId,
        value: Self::Balance,
    ) -> (Self::NegativeImbalance, Self::Balance) {
        if value.is_zero() {
            return (NegativeImbalance::zero(), Zero::zero());
        }

        if Self::total_balance(who).is_zero() {
            return (NegativeImbalance::zero(), value);
        }

        let result = match Self::try_mutate_account_handling_dust(
            who,
            |account, _is_new| -> Result<(Self::NegativeImbalance, Self::Balance), DispatchError> {
                // Best value is the most amount we can slash following liveness rules.
                let actual = value.min(account.free);
                account.free.saturating_reduce(actual);
                let remaining = value.saturating_sub(actual);
                Ok((NegativeImbalance::new(actual), remaining))
            },
        ) {
            Ok((imbalance, remaining)) => {
                Self::deposit_event(Event::Slashed {
                    who: who.clone(),
                    amount: value.saturating_sub(remaining),
                });
                (imbalance, remaining)
            }
            Err(_) => (Self::NegativeImbalance::zero(), value),
        };
        result
    }

    /// Deposit some `value` into the free balance of an existing target account `who`.
    ///
    /// Is a no-op if the `value` to be deposited is zero.
    fn deposit_into_existing(
        who: &<T as Config<I>>::AccountId,
        value: Self::Balance,
    ) -> Result<Self::PositiveImbalance, DispatchError> {
        if value.is_zero() {
            return Ok(PositiveImbalance::zero());
        }

        Self::try_mutate_account_handling_dust(
            who,
            |account, is_new| -> Result<Self::PositiveImbalance, DispatchError> {
                ensure!(!is_new, Error::<T, I>::DeadAccount);
                account.free = account
                    .free
                    .checked_add(&value)
                    .ok_or(ArithmeticError::Overflow)?;
                Self::deposit_event(Event::Deposit {
                    who: who.clone(),
                    amount: value,
                });
                Ok(PositiveImbalance::new(value))
            },
        )
    }

    /// Deposit some `value` into the free balance of `who`, possibly creating a new account.
    ///
    /// This function is a no-op if:
    /// - the `value` to be deposited is zero; or
    /// - the `value` to be deposited is less than the required ED and the account does not yet
    ///   exist; or
    /// - the deposit would necessitate the account to exist and there are no provider references;
    ///   or
    /// - `value` is so large it would cause the balance of `who` to overflow.
    fn deposit_creating(
        who: &<T as Config<I>>::AccountId,
        value: Self::Balance,
    ) -> Self::PositiveImbalance {
        if value.is_zero() {
            return Self::PositiveImbalance::zero();
        }

        Self::try_mutate_account_handling_dust(
            who,
            |account, is_new| -> Result<Self::PositiveImbalance, DispatchError> {
                let ed = T::ExistentialDeposit::get();
                ensure!(value >= ed || !is_new, Error::<T, I>::ExistentialDeposit);

                // defensive only: overflow should never happen, however in case it does, then this
                // operation is a no-op.
                account.free = match account.free.checked_add(&value) {
                    Some(x) => x,
                    None => return Ok(Self::PositiveImbalance::zero()),
                };

                Self::deposit_event(Event::Deposit {
                    who: who.clone(),
                    amount: value,
                });
                Ok(PositiveImbalance::new(value))
            },
        )
        .unwrap_or_else(|_| Self::PositiveImbalance::zero())
    }

    /// Withdraw some free balance from an account, respecting existence requirements.
    ///
    /// Is a no-op if value to be withdrawn is zero.
    fn withdraw(
        who: &<T as Config<I>>::AccountId,
        value: Self::Balance,
        reasons: WithdrawReasons,
        liveness: ExistenceRequirement,
    ) -> result::Result<Self::NegativeImbalance, DispatchError> {
        if value.is_zero() {
            return Ok(NegativeImbalance::zero());
        }

        Self::try_mutate_account_handling_dust(
            who,
            |account, _| -> Result<Self::NegativeImbalance, DispatchError> {
                let new_free_account = account
                    .free
                    .checked_sub(&value)
                    .ok_or(Error::<T, I>::InsufficientBalance)?;

                // bail if we need to keep the account alive and this would kill it.
                let ed = T::ExistentialDeposit::get();
                let would_be_dead = new_free_account < ed;
                let would_kill = would_be_dead && account.free >= ed;
                ensure!(
                    liveness == ExistenceRequirement::AllowDeath || !would_kill,
                    Error::<T, I>::Expendability
                );

                Self::ensure_can_withdraw(who, value, reasons, new_free_account)?;

                account.free = new_free_account;

                Self::deposit_event(Event::Withdraw {
                    who: who.clone(),
                    amount: value,
                });
                Ok(NegativeImbalance::new(value))
            },
        )
    }

    /// Force the new free balance of a target account `who` to some new value `balance`.
    fn make_free_balance_be(
        who: &<T as Config<I>>::AccountId,
        value: Self::Balance,
    ) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
        Self::try_mutate_account_handling_dust(
			who,
			|account,
			 is_new|
			 -> Result<SignedImbalance<Self::Balance, Self::PositiveImbalance>, DispatchError> {
				let ed = T::ExistentialDeposit::get();
				// If we're attempting to set an existing account to less than ED, then
				// bypass the entire operation. It's a no-op if you follow it through, but
				// since this is an instance where we might account for a negative imbalance
				// (in the dust cleaner of set_account) before we account for its actual
				// equal and opposite cause (returned as an Imbalance), then in the
				// instance that there's no other accounts on the system at all, we might
				// underflow the issuance and our arithmetic will be off.
				ensure!(value >= ed || !is_new, Error::<T, I>::ExistentialDeposit);

				let imbalance = if account.free <= value {
					SignedImbalance::Positive(PositiveImbalance::new(value - account.free))
				} else {
					SignedImbalance::Negative(NegativeImbalance::new(account.free - value))
				};
				account.free = value;
				Self::deposit_event(Event::BalanceSet {
					who: who.clone(),
					free: account.free,
				});
				Ok(imbalance)
			},
		)
		.unwrap_or_else(|_| SignedImbalance::Positive(Self::PositiveImbalance::zero()))
    }
}
