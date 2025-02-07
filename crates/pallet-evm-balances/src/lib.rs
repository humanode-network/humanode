//! # EVM Balances Pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure,
    traits::{
        fungible,
        tokens::{DepositConsequence, Fortitude, Preservation, Provenance, WithdrawConsequence},
        ExistenceRequirement, Get, Imbalance, OnUnbalanced, SignedImbalance, StorageVersion,
        StoredMap, WithdrawReasons,
    },
};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{Bounded, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Zero},
    ArithmeticError, DispatchError, DispatchResult, RuntimeDebug, Saturating,
};
use sp_std::{fmt::Debug, result};

mod impl_currency;
mod impl_fungible;

mod account_data;
pub use account_data::{AccountData, Reasons};

mod imbalances;
pub use imbalances::{NegativeImbalance, PositiveImbalance};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use sp_runtime::{
        traits::{AtLeast32BitUnsigned, MaybeDisplay},
        FixedPointOperand,
    };
    use sp_std::fmt::Debug;

    use super::*;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

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

        /// The means of storing the balances of an account.
        type AccountStore: StoredMap<<Self as Config<I>>::AccountId, AccountData<Self::Balance>>;

        /// Handler for the unbalanced reduction when removing a dust account.
        type DustRemoval: OnUnbalanced<NegativeImbalance<Self, I>>;
    }

    /// The total units issued.
    #[pallet::storage]
    #[pallet::whitelist_storage]
    pub type TotalIssuance<T: Config<I>, I: 'static = ()> = StorageValue<_, T::Balance, ValueQuery>;

    /// The total units of outstanding deactivated balance.
    #[pallet::storage]
    #[pallet::whitelist_storage]
    pub type InactiveIssuance<T: Config<I>, I: 'static = ()> =
        StorageValue<_, T::Balance, ValueQuery>;

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
        /// Some amount was minted into an account.
        Minted {
            who: <T as Config<I>>::AccountId,
            amount: T::Balance,
        },
        /// Some amount was burned from an account.
        Burned {
            who: <T as Config<I>>::AccountId,
            amount: T::Balance,
        },
        /// Some amount was suspended from an account (it can be restored later).
        Suspended {
            who: <T as Config<I>>::AccountId,
            amount: T::Balance,
        },
        /// Some amount was restored into an account.
        Restored {
            who: <T as Config<I>>::AccountId,
            amount: T::Balance,
        },
        /// Total issuance was increased by `amount`, creating a credit to be balanced.
        Issued { amount: T::Balance },
        /// Total issuance was decreased by `amount`, creating a debt to be balanced.
        Rescinded { amount: T::Balance },
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Account liquidity restrictions prevent withdrawal.
        LiquidityRestrictions,
        /// Balance too low to send value.
        InsufficientBalance,
        /// Value too low to create account due to existential deposit.
        ExistentialDeposit,
        /// Transfer/payment would kill account.
        Expendability,
        /// Beneficiary account must pre-exist.
        DeadAccount,
    }
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    /// Get the free balance of an account.
    pub fn free_balance(
        who: impl sp_std::borrow::Borrow<<T as Config<I>>::AccountId>,
    ) -> T::Balance {
        Self::account(who.borrow()).free
    }

    /// Get all data information for an account.
    fn account(who: &<T as Config<I>>::AccountId) -> AccountData<T::Balance> {
        T::AccountStore::get(who)
    }

    /// Mutate an account to some new value, or delete it entirely with `None`. Will enforce
    /// `ExistentialDeposit` law, annulling the account as needed.
    ///
    /// It returns the result from the closure. Any dust is handled through the low-level
    /// `fungible::Unbalanced` trap-door for legacy dust management.
    ///
    /// NOTE: Doesn't do any preparatory work for creating a new account, so should only be used
    /// when it is known that the account already exists.
    ///
    /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    /// the caller will do this.
    pub(crate) fn try_mutate_account_handling_dust<R, E: From<DispatchError>>(
        who: &<T as Config<I>>::AccountId,
        f: impl FnOnce(&mut AccountData<T::Balance>, bool) -> Result<R, E>,
    ) -> Result<R, E> {
        let (r, maybe_dust) = Self::try_mutate_account(who, f)?;
        if let Some(dust) = maybe_dust {
            <Self as fungible::Unbalanced<_>>::handle_raw_dust(dust);
        }
        Ok(r)
    }

    /// Mutate an account to some new value, or delete it entirely with `None`. Will enforce
    /// `ExistentialDeposit` law, annulling the account as needed.
    ///
    /// It returns both the result from the closure, and an optional amount of dust
    /// which should be handled once it is known that all nested mutates that could affect
    /// storage items what the dust handler touches have completed.
    ///
    /// NOTE: Doesn't do any preparatory work for creating a new account, so should only be used
    /// when it is known that the account already exists.
    ///
    /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    /// the caller will do this.
    pub(crate) fn mutate_account<R>(
        who: &<T as Config<I>>::AccountId,
        f: impl FnOnce(&mut AccountData<T::Balance>) -> R,
    ) -> Result<(R, Option<T::Balance>), DispatchError> {
        Self::try_mutate_account(who, |a, _| -> Result<R, DispatchError> { Ok(f(a)) })
    }

    /// Mutate an account to some new value, or delete it entirely with `None`. Will enforce
    /// `ExistentialDeposit` law, annulling the account as needed. This will do nothing if the
    /// result of `f` is an `Err`.
    ///
    /// It returns both the result from the closure, and an optional amount of dust
    /// which should be handled once it is known that all nested mutates that could affect
    /// storage items what the dust handler touches have completed.
    ///
    /// NOTE: Doesn't do any preparatory work for creating a new account, so should only be used
    /// when it is known that the account already exists.
    ///
    /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    /// the caller will do this.
    fn try_mutate_account<R, E: From<DispatchError>>(
        who: &<T as Config<I>>::AccountId,
        f: impl FnOnce(&mut AccountData<T::Balance>, bool) -> Result<R, E>,
    ) -> Result<(R, Option<T::Balance>), E> {
        let result = T::AccountStore::try_mutate_exists(who, |maybe_account| {
            let is_new = maybe_account.is_none();
            let mut account = maybe_account.take().unwrap_or_default();
            f(&mut account, is_new).map(move |result| {
                let maybe_endowed = if is_new { Some(account.free) } else { None };

                // Handle any steps needed after mutating an account.
                //
                // This includes DustRemoval unbalancing, in the case than the `new` account's total
                // balance is non-zero but below ED.
                //
                // Updates `maybe_account` to `Some` iff the account has sufficient balance.
                // Evaluates `maybe_dust`, which is `Some` containing the dust to be dropped, iff
                // some dust should be dropped.
                //
                // We should never be dropping if reserved is non-zero. Reserved being non-zero
                // should imply that we have a consumer ref, so this is economically safe.
                let maybe_dust = if account.free < T::ExistentialDeposit::get() {
                    if account.free.is_zero() {
                        None
                    } else {
                        Some(account.free)
                    }
                } else {
                    assert!(account.free.is_zero() || account.free >= T::ExistentialDeposit::get());
                    *maybe_account = Some(account);
                    None
                };

                (maybe_endowed, maybe_dust, result)
            })
        });
        result.map(|(maybe_endowed, maybe_dust, result)| {
            if let Some(endowed) = maybe_endowed {
                Self::deposit_event(Event::Endowed {
                    account: who.clone(),
                    free_balance: endowed,
                });
            }
            if let Some(amount) = maybe_dust {
                Pallet::<T, I>::deposit_event(Event::DustLost {
                    account: who.clone(),
                    amount,
                });
            }
            (result, maybe_dust)
        })
    }
}
