//! A substrate pallet provides additional functionality for handling non native accounts and balances.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode, MaxEncodedLen};
use frame_support::traits::{
    tokens::{DepositConsequence, WithdrawConsequence},
    Get, Imbalance, OnUnbalanced, StorageVersion, WithdrawReasons,
};
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{CheckedAdd, CheckedSub, Zero},
    DispatchError, RuntimeDebug, Saturating,
};

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
    /// How much this account's balance can be reduced for the given `reasons`.
    fn usable(&self, reasons: Reasons) -> Balance {
        self.free.saturating_sub(self.frozen(reasons))
    }

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
