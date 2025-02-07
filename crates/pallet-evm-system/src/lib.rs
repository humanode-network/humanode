//! # EVM System Pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, FullCodec, MaxEncodedLen};
use frame_support::traits::StoredMap;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{One, Zero},
    DispatchError, RuntimeDebug,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

/// Account information.
#[derive(Clone, Eq, PartialEq, Default, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct AccountInfo<Index, AccountData> {
    /// The number of transactions this account has sent.
    pub nonce: Index,
    /// The additional data that belongs to this account. Used to store the balance(s) in a lot of
    /// chains.
    pub data: AccountData,
}

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use sp_runtime::traits::{AtLeast32Bit, MaybeDisplay};
    use sp_std::fmt::Debug;

    use super::*;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The user account identifier type.
        type AccountId: Parameter
            + Member
            + MaybeSerializeDeserialize
            + Debug
            + MaybeDisplay
            + Ord
            + MaxEncodedLen;

        /// Account index (aka nonce) type. This stores the number of previous transactions
        /// associated with a sender account.
        type Index: Parameter
            + Member
            + MaybeSerializeDeserialize
            + Debug
            + Default
            + MaybeDisplay
            + AtLeast32Bit
            + Copy
            + MaxEncodedLen;

        /// Data to be associated with an account (other than nonce/transaction counter, which this
        /// pallet does regardless).
        type AccountData: Member + FullCodec + Clone + Default + TypeInfo + MaxEncodedLen;

        /// Handler for when a new account has just been created.
        type OnNewAccount: OnNewAccount<<Self as Config>::AccountId>;

        /// A function that is invoked when an account has been determined to be dead.
        ///
        /// All resources should be cleaned up associated with the given account.
        type OnKilledAccount: OnKilledAccount<<Self as Config>::AccountId>;
    }

    /// The full account information for a particular account ID.
    #[pallet::storage]
    pub type Account<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        <T as Config>::AccountId,
        AccountInfo<<T as Config>::Index, <T as Config>::AccountData>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new account was created.
        NewAccount { account: <T as Config>::AccountId },
        /// An account was reaped.
        KilledAccount { account: <T as Config>::AccountId },
    }
}

/// The outcome of the account creation operation.
#[derive(Eq, PartialEq, RuntimeDebug)]
pub enum AccountCreationOutcome {
    /// Account was created.
    Created,
    /// Account already exists in the system, so action was taken.
    AlreadyExists,
}

impl<T: Config> Pallet<T> {
    /// Check the account existence.
    pub fn account_exists(who: &<T as Config>::AccountId) -> bool {
        Account::<T>::contains_key(who)
    }

    /// An account is being created.
    fn on_created_account(who: <T as Config>::AccountId) {
        <T as Config>::OnNewAccount::on_new_account(&who);
        Self::deposit_event(Event::NewAccount { account: who });
    }

    /// Do anything that needs to be done after an account has been killed.
    fn on_killed_account(who: <T as Config>::AccountId) {
        <T as Config>::OnKilledAccount::on_killed_account(&who);
        Self::deposit_event(Event::KilledAccount { account: who });
    }

    /// Retrieve the account transaction counter from storage.
    pub fn account_nonce(who: &<T as Config>::AccountId) -> <T as Config>::Index {
        Account::<T>::get(who).nonce
    }

    /// Increment a particular account's nonce by 1.
    pub fn inc_account_nonce(who: &<T as Config>::AccountId) {
        let is_new_account = Account::<T>::mutate_exists(who, |maybe_account| {
            let is_new_account = maybe_account.is_none();

			let account = maybe_account.get_or_insert_with(Default::default);
			account.nonce += <T as Config>::Index::one();

            is_new_account
        });

        // Meaning that account is being created.
        if is_new_account {
            Self::on_created_account(who.clone());
        }
    }

    /// Create an account.
    pub fn create_account(who: &<T as Config>::AccountId) -> AccountCreationOutcome {
        if Self::account_exists(who) {
            return AccountCreationOutcome::AlreadyExists;
        }

        Account::<T>::insert(who.clone(), AccountInfo::<_, _>::default());
        Self::on_created_account(who.clone());
        AccountCreationOutcome::Created
    }
}

impl<T: Config> StoredMap<<T as Config>::AccountId, <T as Config>::AccountData> for Pallet<T> {
    fn get(k: &<T as Config>::AccountId) -> <T as Config>::AccountData {
        Account::<T>::get(k).data
    }

    fn try_mutate_exists<R, E: From<DispatchError>>(
        k: &<T as Config>::AccountId,
        f: impl FnOnce(&mut Option<<T as Config>::AccountData>) -> Result<R, E>,
    ) -> Result<R, E> {
        let (mut maybe_account_data, nonce, was_providing) = if Self::account_exists(k) {
            let account_info = Account::<T>::get(k);
            (Some(account_info.data), account_info.nonce, true)
        } else {
            (None, <T as Config>::Index::zero(), false)
        };

        let result = f(&mut maybe_account_data)?;

        match (maybe_account_data, was_providing) {
            (Some(data), false) => {
                Account::<T>::mutate(k, |a| a.data = data);
                Self::on_created_account(k.clone());
            }
            (Some(data), true) => {
                Account::<T>::mutate(k, |a| a.data = data);
            }
            (None, true) => {
                if nonce != <T as Config>::Index::zero() {
                    Account::<T>::mutate(k, |a| a.data = Default::default());
                } else {
                    Account::<T>::remove(k);
                    Self::on_killed_account(k.clone());
                }
            }
            (None, false) => {
                // Do nothing.
            }
        }

        Ok(result)
    }
}

impl<T: Config> fp_evm::AccountProvider for Pallet<T> {
    type AccountId = <T as Config>::AccountId;
    type Index = <T as Config>::Index;

    fn create_account(who: &Self::AccountId) {
        let _ = Self::create_account(who);
    }

    fn on_account_self_destruct(_who: &Self::AccountId) {
        // Do nothing as account state shouldn't be modified.
    }

    fn account_nonce(who: &Self::AccountId) -> Self::Index {
        Self::account_nonce(who)
    }

    fn inc_account_nonce(who: &Self::AccountId) {
        Self::inc_account_nonce(who);
    }
}

/// Interface to handle account creation.
pub trait OnNewAccount<AccountId> {
    /// A new account `who` has been registered.
    fn on_new_account(who: &AccountId);
}

impl<AccountId> OnNewAccount<AccountId> for () {
    fn on_new_account(_who: &AccountId) {}
}

/// Interface to handle account killing.
pub trait OnKilledAccount<AccountId> {
    /// The account with the given id was reaped.
    fn on_killed_account(who: &AccountId);
}

impl<AccountId> OnKilledAccount<AccountId> for () {
    fn on_killed_account(_who: &AccountId) {}
}
