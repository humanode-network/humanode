//! A substrate pallet provides additional functionality for handling non native accounts and balances.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure,
    traits::{
        fungible,
        tokens::{DepositConsequence, WithdrawConsequence},
        Currency, ExistenceRequirement,
        ExistenceRequirement::AllowDeath,
        Get, Imbalance, OnUnbalanced, SignedImbalance, StorageVersion, WithdrawReasons,
    },
};
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{Bounded, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Zero},
    ArithmeticError, DispatchError, DispatchResult, RuntimeDebug, Saturating,
};
use sp_std::{cmp, fmt::Debug, result};

mod imbalances;
pub use imbalances::{NegativeImbalance, PositiveImbalance};

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

/// Simplified reasons for withdrawing balance.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Reasons {
    /// Paying system transaction fees.
    Fee = 0,
    /// Any reason other than paying system transaction fees.
    Misc = 1,
    /// Any reason at all.
    All = 2,
}

impl From<WithdrawReasons> for Reasons {
    fn from(r: WithdrawReasons) -> Reasons {
        if r == WithdrawReasons::TRANSACTION_PAYMENT {
            Reasons::Fee
        } else if r.contains(WithdrawReasons::TRANSACTION_PAYMENT) {
            Reasons::All
        } else {
            Reasons::Misc
        }
    }
}

/// All balance information for an account.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AccountData<Balance> {
    /// Non-reserved part of the balance. There may still be restrictions on this, but it is the
    /// total pool what may in principle be transferred, reserved and used for tipping.
    ///
    /// This is the only balance that matters in terms of most operations on tokens. It
    /// alone is used to determine the balance when in the contract execution environment.
    pub free: Balance,
    /// Balance which is reserved and may not be used at all.
    ///
    /// This can still get slashed, but gets slashed last of all.
    ///
    /// This balance is a 'reserve' balance that other subsystems use in order to set aside tokens
    /// that are still 'owned' by the account holder, but which are suspendable.
    /// This includes named reserve and unnamed reserve.
    pub reserved: Balance,
    /// The amount that `free` may not drop below when withdrawing for *anything except transaction
    /// fee payment*.
    pub misc_frozen: Balance,
    /// The amount that `free` may not drop below when withdrawing specifically for transaction
    /// fee payment.
    pub fee_frozen: Balance,
}

impl<Balance: Saturating + Copy + Ord> AccountData<Balance> {
    /// The amount that this account's free balance may not be reduced beyond for the given
    /// `reasons`.
    fn frozen(&self, reasons: Reasons) -> Balance {
        match reasons {
            Reasons::All => self.misc_frozen.max(self.fee_frozen),
            Reasons::Misc => self.misc_frozen,
            Reasons::Fee => self.fee_frozen,
        }
    }

    /// The total balance in this account including any that is reserved and ignoring any frozen.
    fn total(&self) -> Balance {
        self.free.saturating_add(self.reserved)
    }
}

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::unused_unit
)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use sp_runtime::{
        traits::{AtLeast32BitUnsigned, MaybeDisplay},
        FixedPointOperand,
    };
    use sp_std::fmt::Debug;

    use super::*;

    /// Configuration trait of this pallet.
    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self, I>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The user account identifier type.
        type AccountId: Parameter
            + Member
            + MaybeSerializeDeserialize
            + Debug
            + MaybeDisplay
            + Ord
            + MaxEncodedLen;

        /// The balance of an account.
        type Balance: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Codec
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + MaxEncodedLen
            + TypeInfo
            + FixedPointOperand;

        /// The minimum amount required to keep an account open.
        #[pallet::constant]
        type ExistentialDeposit: Get<Self::Balance>;

        /// Handler for the unbalanced reduction when removing a dust account.
        type DustRemoval: OnUnbalanced<NegativeImbalance<Self, I>>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

    /// The total units issued.
    #[pallet::storage]
    #[pallet::getter(fn total_issuance)]
    #[pallet::whitelist_storage]
    pub type TotalIssuance<T: Config<I>, I: 'static = ()> = StorageValue<_, T::Balance, ValueQuery>;

    /// The total units of outstanding deactivated balance.
    #[pallet::storage]
    #[pallet::getter(fn inactive_issuance)]
    #[pallet::whitelist_storage]
    pub type InactiveIssuance<T: Config<I>, I: 'static = ()> =
        StorageValue<_, T::Balance, ValueQuery>;

    /// The full account balance information for a particular account ID.
    #[pallet::storage]
    #[pallet::getter(fn account_store)]
    pub type AccountStore<T: Config<I>, I: 'static = ()> = StorageMap<
        _,
        Blake2_128Concat,
        <T as Config<I>>::AccountId,
        AccountData<T::Balance>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// An account was created with some free balance.
        Endowed {
            account: <T as Config<I>>::AccountId,
            free_balance: T::Balance,
        },
        /// An account was removed whose balance was non-zero but below ExistentialDeposit,
        /// resulting in an outright loss.
        DustLost {
            account: <T as Config<I>>::AccountId,
            amount: T::Balance,
        },
        /// Transfer succeeded.
        Transfer {
            from: <T as Config<I>>::AccountId,
            to: <T as Config<I>>::AccountId,
            amount: T::Balance,
        },
        /// A balance was set by root.
        BalanceSet {
            who: <T as Config<I>>::AccountId,
            free: T::Balance,
            reserved: T::Balance,
        },
        /// Some balance was reserved (moved from free to reserved).
        Reserved {
            who: <T as Config<I>>::AccountId,
            amount: T::Balance,
        },
        /// Some amount was deposited (e.g. for transaction fees).
        Deposit {
            who: <T as Config<I>>::AccountId,
            amount: T::Balance,
        },
        /// Some amount was withdrawn from the account (e.g. for transaction fees).
        Withdraw {
            who: <T as Config<I>>::AccountId,
            amount: T::Balance,
        },
        /// Some amount was removed from the account (e.g. for misbehavior).
        Slashed {
            who: <T as Config<I>>::AccountId,
            amount: T::Balance,
        },
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Account liquidity restrictions prevent withdrawal
        LiquidityRestrictions,
        /// Balance too low to send value.
        InsufficientBalance,
        /// Value too low to create account due to existential deposit
        ExistentialDeposit,
        /// Transfer/payment would kill account
        KeepAlive,
        /// A vesting schedule already exists for this account
        ExistingVestingSchedule,
        /// Beneficiary account must pre-exist
        DeadAccount,
        /// Number of named reserves exceed MaxReserves
        TooManyReserves,
    }
}

/// Removes a dust account whose balance was non-zero but below `ExistentialDeposit`.
pub struct DustCleaner<T: Config<I>, I: 'static = ()>(
    Option<(<T as Config<I>>::AccountId, NegativeImbalance<T, I>)>,
);

impl<T: Config<I>, I: 'static> Drop for DustCleaner<T, I> {
    fn drop(&mut self) {
        if let Some((who, dust)) = self.0.take() {
            Pallet::<T, I>::deposit_event(Event::DustLost {
                account: who,
                amount: dust.peek(),
            });
            T::DustRemoval::on_unbalanced(dust);
        }
    }
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    /// Get all balance information for an account.
    fn account(who: &<T as Config<I>>::AccountId) -> AccountData<T::Balance> {
        <AccountStore<T, I>>::get(who)
    }

    /// Mutate an account to some new value, or delete it entirely with `None`. Will enforce
    /// `ExistentialDeposit` law, annulling the account as needed. This will do nothing if the
    /// result of `f` is an `Err`.
    ///
    /// NOTE: Doesn't do any preparatory work for creating a new account, so should only be used
    /// when it is known that the account already exists.
    ///
    /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    /// the caller will do this.
    fn try_mutate_account<R, E: From<DispatchError>>(
        who: &<T as Config<I>>::AccountId,
        f: impl FnOnce(&mut AccountData<T::Balance>, bool) -> Result<R, E>,
    ) -> Result<R, E> {
        Self::try_mutate_account_with_dust(who, f).map(|(result, dust_cleaner)| {
            drop(dust_cleaner);
            result
        })
    }

    /// Mutate an account to some new value, or delete it entirely with `None`. Will enforce
    /// `ExistentialDeposit` law, annulling the account as needed. This will do nothing if the
    /// result of `f` is an `Err`.
    ///
    /// It returns both the result from the closure, and an optional `DustCleaner` instance which
    /// should be dropped once it is known that all nested mutates that could affect storage items
    /// what the dust handler touches have completed.
    ///
    /// NOTE: Doesn't do any preparatory work for creating a new account, so should only be used
    /// when it is known that the account already exists.
    ///
    /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    /// the caller will do this.
    fn try_mutate_account_with_dust<R, E: From<DispatchError>>(
        who: &<T as Config<I>>::AccountId,
        f: impl FnOnce(&mut AccountData<T::Balance>, bool) -> Result<R, E>,
    ) -> Result<(R, DustCleaner<T, I>), E> {
        let result = <AccountStore<T, I>>::try_mutate_exists(who, |maybe_account| {
            let is_new = maybe_account.is_none();
            let mut account = maybe_account.take().unwrap_or_default();
            f(&mut account, is_new).map(move |result| {
                let maybe_endowed = if is_new { Some(account.free) } else { None };
                let maybe_account_maybe_dust = Self::post_mutation(who, account);
                *maybe_account = maybe_account_maybe_dust.0;
                (maybe_endowed, maybe_account_maybe_dust.1, result)
            })
        });
        result.map(|(maybe_endowed, maybe_dust, result)| {
            if let Some(endowed) = maybe_endowed {
                Self::deposit_event(Event::Endowed {
                    account: who.clone(),
                    free_balance: endowed,
                });
            }
            let dust_cleaner = DustCleaner(maybe_dust.map(|dust| (who.clone(), dust)));
            (result, dust_cleaner)
        })
    }

    /// Handles any steps needed after mutating an account.
    ///
    /// This includes `DustRemoval` unbalancing, in the case than the `new` account's total balance
    /// is non-zero but below ED.
    ///
    /// Returns two values:
    /// - `Some` containing the the `new` account, iff the account has sufficient balance.
    /// - `Some` containing the dust to be dropped, iff some dust should be dropped.
    fn post_mutation(
        _who: &<T as Config<I>>::AccountId,
        new: AccountData<T::Balance>,
    ) -> (
        Option<AccountData<T::Balance>>,
        Option<NegativeImbalance<T, I>>,
    ) {
        let total = new.total();
        if total < T::ExistentialDeposit::get() {
            if total.is_zero() {
                (None, None)
            } else {
                (None, Some(NegativeImbalance::new(total)))
            }
        } else {
            (Some(new), None)
        }
    }

    fn deposit_consequence(
        _who: &<T as Config<I>>::AccountId,
        amount: T::Balance,
        account: &AccountData<T::Balance>,
        mint: bool,
    ) -> DepositConsequence {
        if amount.is_zero() {
            return DepositConsequence::Success;
        }

        if mint && TotalIssuance::<T, I>::get().checked_add(&amount).is_none() {
            return DepositConsequence::Overflow;
        }

        let new_total_balance = match account.total().checked_add(&amount) {
            Some(x) => x,
            None => return DepositConsequence::Overflow,
        };

        if new_total_balance < T::ExistentialDeposit::get() {
            return DepositConsequence::BelowMinimum;
        }

        // NOTE: We assume that we are a provider, so don't need to do any checks in the
        // case of account creation.

        DepositConsequence::Success
    }

    fn withdraw_consequence(
        who: &<T as Config<I>>::AccountId,
        amount: T::Balance,
        account: &AccountData<T::Balance>,
    ) -> WithdrawConsequence<T::Balance> {
        if amount.is_zero() {
            return WithdrawConsequence::Success;
        }

        if TotalIssuance::<T, I>::get().checked_sub(&amount).is_none() {
            return WithdrawConsequence::Underflow;
        }

        let new_total_balance = match account.total().checked_sub(&amount) {
            Some(x) => x,
            None => return WithdrawConsequence::NoFunds,
        };

        // Provider restriction - total account balance cannot be reduced to zero if it cannot
        // sustain the loss of a provider reference.
        // NOTE: This assumes that the pallet is a provider (which is true). Is this ever changes,
        // then this will need to adapt accordingly.
        let ed = T::ExistentialDeposit::get();
        let success = if new_total_balance < ed {
            // ATTENTION. CHECK.
            // if frame_system::Pallet::<T>::can_dec_provider(who) {
            //     WithdrawConsequence::ReducedToZero(new_total_balance)
            // } else {
            //     return WithdrawConsequence::WouldDie;
            // }
            WithdrawConsequence::ReducedToZero(new_total_balance)
        } else {
            WithdrawConsequence::Success
        };

        // Enough free funds to have them be reduced.
        let new_free_balance = match account.free.checked_sub(&amount) {
            Some(b) => b,
            None => return WithdrawConsequence::NoFunds,
        };

        // Eventual free funds must be no less than the frozen balance.
        let min_balance = account.frozen(Reasons::All);
        if new_free_balance < min_balance {
            return WithdrawConsequence::Frozen;
        }

        success
    }
}

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

    // Check if `value` amount of free balance can be slashed from `who`.
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
        <Self as fungible::Inspect<<T as Config<I>>::AccountId>>::active_issuance()
    }

    fn deactivate(amount: Self::Balance) {
        InactiveIssuance::<T, I>::mutate(|b| b.saturating_accrue(amount));
    }

    fn reactivate(amount: Self::Balance) {
        InactiveIssuance::<T, I>::mutate(|b| b.saturating_reduce(amount));
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

    // Ensure that an account can withdraw from their free balance given any existing withdrawal
    // restrictions like locks and vesting balance.
    // Is a no-op if amount to be withdrawn is zero.
    //
    // # <weight>
    // Despite iterating over a list of locks, they are limited by the number of
    // lock IDs, which means the number of runtime pallets that intend to use and create locks.
    // # </weight>
    fn ensure_can_withdraw(
        who: &<T as Config<I>>::AccountId,
        amount: T::Balance,
        reasons: WithdrawReasons,
        new_balance: T::Balance,
    ) -> DispatchResult {
        if amount.is_zero() {
            return Ok(());
        }
        let min_balance = Self::account(who).frozen(reasons.into());
        ensure!(
            new_balance >= min_balance,
            Error::<T, I>::LiquidityRestrictions
        );
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

        Self::try_mutate_account_with_dust(
            dest,
            |to_account, _| -> Result<DustCleaner<T, I>, DispatchError> {
                Self::try_mutate_account_with_dust(
                    transactor,
                    |from_account, _| -> DispatchResult {
                        from_account.free = from_account
                            .free
                            .checked_sub(&value)
                            .ok_or(Error::<T, I>::InsufficientBalance)?;

                        // NOTE: total stake being stored in the same type means that this could
                        // never overflow but better to be safe than sorry.
                        to_account.free = to_account
                            .free
                            .checked_add(&value)
                            .ok_or(ArithmeticError::Overflow)?;

                        let ed = T::ExistentialDeposit::get();
                        ensure!(to_account.total() >= ed, Error::<T, I>::ExistentialDeposit);

                        Self::ensure_can_withdraw(
                            transactor,
                            value,
                            WithdrawReasons::TRANSFER,
                            from_account.free,
                        )
                        .map_err(|_| Error::<T, I>::LiquidityRestrictions)?;

                        // TODO: This is over-conservative. There may now be other providers, and
                        // this pallet may not even be a provider.
                        let allow_death = existence_requirement == ExistenceRequirement::AllowDeath;
                        // ATTENTION. CHECK.
                        // let allow_death =
                        //     allow_death && system::Pallet::<T>::can_dec_provider(transactor);
                        ensure!(
                            allow_death || from_account.total() >= ed,
                            Error::<T, I>::KeepAlive
                        );

                        Ok(())
                    },
                )
                .map(|(_, maybe_dust_cleaner)| maybe_dust_cleaner)
            },
        )?;

        // Emit transfer event.
        Self::deposit_event(Event::Transfer {
            from: transactor.clone(),
            to: dest.clone(),
            amount: value,
        });

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

        for attempt in 0..2 {
            match Self::try_mutate_account(
				who,
				|account,
				 _is_new|
				 -> Result<(Self::NegativeImbalance, Self::Balance), DispatchError> {
					// Best value is the most amount we can slash following liveness rules.
					let best_value = match attempt {
						// First attempt we try to slash the full amount, and see if liveness issues
						// happen.
						0 => value,
						// If acting as a critical provider (i.e. first attempt failed), then slash
						// as much as possible while leaving at least at ED.
						_ => value.min(
							(account.free + account.reserved)
								.saturating_sub(T::ExistentialDeposit::get()),
						),
					};

					let free_slash = cmp::min(account.free, best_value);
					account.free -= free_slash; // Safe because of above check
					let remaining_slash = best_value - free_slash; // Safe because of above check

					if !remaining_slash.is_zero() {
						// If we have remaining slash, take it from reserved balance.
						let reserved_slash = cmp::min(account.reserved, remaining_slash);
						account.reserved -= reserved_slash; // Safe because of above check
						Ok((
							NegativeImbalance::new(free_slash + reserved_slash),
							value - free_slash - reserved_slash, /* Safe because value is gt or
							                                      * eq total slashed */
						))
					} else {
						// Else we are done!
						Ok((
							NegativeImbalance::new(free_slash),
							value - free_slash, // Safe because value is gt or eq to total slashed
						))
					}
				},
			) {
				Ok((imbalance, not_slashed)) => {
					Self::deposit_event(Event::Slashed {
						who: who.clone(),
						amount: value.saturating_sub(not_slashed),
					});
					return (imbalance, not_slashed)
				},
				Err(_) => (),
			}
        }

        // Should never get here. But we'll be defensive anyway.
        (Self::NegativeImbalance::zero(), value)
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

        Self::try_mutate_account(
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

        Self::try_mutate_account(
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

        Self::try_mutate_account(
            who,
            |account, _| -> Result<Self::NegativeImbalance, DispatchError> {
                let new_free_account = account
                    .free
                    .checked_sub(&value)
                    .ok_or(Error::<T, I>::InsufficientBalance)?;

                // bail if we need to keep the account alive and this would kill it.
                let ed = T::ExistentialDeposit::get();
                let would_be_dead = new_free_account + account.reserved < ed;
                let would_kill = would_be_dead && account.free + account.reserved >= ed;
                ensure!(
                    liveness == AllowDeath || !would_kill,
                    Error::<T, I>::KeepAlive
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
        Self::try_mutate_account(
			who,
			|account,
			 is_new|
			 -> Result<SignedImbalance<Self::Balance, Self::PositiveImbalance>, DispatchError> {
				let ed = T::ExistentialDeposit::get();
				let total = value.saturating_add(account.reserved);
				// If we're attempting to set an existing account to less than ED, then
				// bypass the entire operation. It's a no-op if you follow it through, but
				// since this is an instance where we might account for a negative imbalance
				// (in the dust cleaner of set_account) before we account for its actual
				// equal and opposite cause (returned as an Imbalance), then in the
				// instance that there's no other accounts on the system at all, we might
				// underflow the issuance and our arithmetic will be off.
				ensure!(total >= ed || !is_new, Error::<T, I>::ExistentialDeposit);

				let imbalance = if account.free <= value {
					SignedImbalance::Positive(PositiveImbalance::new(value - account.free))
				} else {
					SignedImbalance::Negative(NegativeImbalance::new(account.free - value))
				};
				account.free = value;
				Self::deposit_event(Event::BalanceSet {
					who: who.clone(),
					free: account.free,
					reserved: account.reserved,
				});
				Ok(imbalance)
			},
		)
		.unwrap_or_else(|_| SignedImbalance::Positive(Self::PositiveImbalance::zero()))
    }
}

impl<T: Config<I>, I: 'static> fungible::Inspect<<T as Config<I>>::AccountId> for Pallet<T, I> {
    type Balance = T::Balance;

    fn total_issuance() -> Self::Balance {
        TotalIssuance::<T, I>::get()
    }
    fn active_issuance() -> Self::Balance {
        TotalIssuance::<T, I>::get().saturating_sub(InactiveIssuance::<T, I>::get())
    }
    fn minimum_balance() -> Self::Balance {
        T::ExistentialDeposit::get()
    }
    fn balance(who: &<T as Config<I>>::AccountId) -> Self::Balance {
        Self::account(who).total()
    }
    fn reducible_balance(who: &<T as Config<I>>::AccountId, keep_alive: bool) -> Self::Balance {
        let a = Self::account(who);
        // Liquid balance is what is neither reserved nor locked/frozen.
        let liquid = a.free.saturating_sub(a.fee_frozen.max(a.misc_frozen));
        // ATTENTION. CHECK.
        // if frame_system::Pallet::<T>::can_dec_provider(who) && !keep_alive {
        //     liquid
        if !keep_alive {
            liquid
        } else {
            // `must_remain_to_exist` is the part of liquid balance which must remain to keep total
            // over ED.
            let must_remain_to_exist =
                T::ExistentialDeposit::get().saturating_sub(a.total() - liquid);
            liquid.saturating_sub(must_remain_to_exist)
        }
    }
    fn can_deposit(
        who: &<T as Config<I>>::AccountId,
        amount: Self::Balance,
        mint: bool,
    ) -> DepositConsequence {
        Self::deposit_consequence(who, amount, &Self::account(who), mint)
    }
    fn can_withdraw(
        who: &<T as Config<I>>::AccountId,
        amount: Self::Balance,
    ) -> WithdrawConsequence<Self::Balance> {
        Self::withdraw_consequence(who, amount, &Self::account(who))
    }
}
