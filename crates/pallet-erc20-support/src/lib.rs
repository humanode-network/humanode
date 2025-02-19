//! A substrate pallet that exposes currency instance using the ERC20 interface standard.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    sp_runtime::DispatchResult,
    sp_std::ops::Sub,
    storage::with_storage_layer,
    traits::{Currency, StorageVersion},
};
#[cfg(feature = "try-runtime")]
use frame_support::{sp_runtime::TryRuntimeError, sp_std::vec::Vec};
pub use pallet::*;

mod migrations;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Metadata of an ERC20 token.
pub trait Metadata {
    /// Returns the name of the token.
    fn name() -> &'static str;

    /// Returns the symbol of the token.
    fn symbol() -> &'static str;

    /// Returns the decimals places of the token.
    fn decimals() -> u8;
}

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

/// Utility alias for easy access to the [`Config::AccountId`].
type AccountIdOf<T, I> = <T as Config<I>>::AccountId;

/// Utility alias for easy access to the [`Currency::Balance`] of the [`Config::Currency`] type.
type BalanceOf<T, I> = <<T as Config<I>>::Currency as Currency<AccountIdOf<T, I>>>::Balance;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {

    use frame_support::{pallet_prelude::*, sp_runtime::traits::MaybeDisplay, sp_std::fmt::Debug};
    use frame_system::pallet_prelude::*;

    use super::*;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T, I = ()>(_);

    /// Configuration trait of this pallet.
    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        /// The user account identifier type.
        type AccountId: Parameter
            + Member
            + MaybeSerializeDeserialize
            + Debug
            + MaybeDisplay
            + Ord
            + MaxEncodedLen;

        /// The currency to be exposed as ERC20 token.
        type Currency: Currency<AccountIdOf<Self, I>>;

        /// Allowance type.
        type Allowance: Parameter
            + Default
            + Copy
            + From<BalanceOf<Self, I>>
            + Sub<Output = Self::Allowance>
            + PartialOrd
            + MaxEncodedLen;

        /// Interface into ERC20 metadata implementation.
        type Metadata: Metadata;
    }

    /// ERC20-style approvals data.
    /// (Owner => Allowed => Amount).
    #[pallet::storage]
    #[pallet::getter(fn approvals)]
    pub type Approvals<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AccountIdOf<T, I>,
        Blake2_128Concat,
        AccountIdOf<T, I>,
        T::Allowance,
        ValueQuery,
    >;

    /// Possible errors.
    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Spender can't transfer tokens more than allowed.
        SpendMoreThanAllowed,
    }

    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
        fn on_runtime_upgrade() -> Weight {
            let mut weight = T::DbWeight::get().reads(1);

            if StorageVersion::get::<Pallet<T, I>>() == 0 {
                weight.saturating_accrue(migrations::v1::migrate::<T, I>());
                StorageVersion::new(1).put::<Pallet<T, I>>();
                weight.saturating_accrue(T::DbWeight::get().writes(1));
            }

            weight
        }

        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
            Ok(Vec::new())
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
            ensure!(
                <Pallet<T, I>>::on_chain_storage_version()
                    == <Pallet<T, I>>::current_storage_version(),
                "the current storage version and onchain storage version should be the same"
            );
            Ok(())
        }
    }
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    /// Returns the amount of tokens in existence.
    pub fn total_supply() -> BalanceOf<T, I> {
        T::Currency::total_issuance()
    }

    /// Returns the amount of tokens owned by provided account.
    pub fn balance_of(owner: &AccountIdOf<T, I>) -> BalanceOf<T, I> {
        T::Currency::total_balance(owner)
    }

    /// Returns the remaining number of tokens that spender will be allowed to spend on behalf of
    /// owner. This is zero by default.
    pub fn allowance(owner: &AccountIdOf<T, I>, spender: &AccountIdOf<T, I>) -> T::Allowance {
        <Approvals<T, I>>::get(owner, spender)
    }

    /// Sets amount as the allowance of spender over the caller’s tokens.
    pub fn approve(owner: AccountIdOf<T, I>, spender: AccountIdOf<T, I>, amount: T::Allowance) {
        <Approvals<T, I>>::insert(owner, spender, amount);
    }

    /// Moves amount tokens from the caller’s account to recipient.
    pub fn transfer(
        caller: AccountIdOf<T, I>,
        recipient: AccountIdOf<T, I>,
        amount: BalanceOf<T, I>,
    ) -> DispatchResult {
        with_storage_layer(move || {
            T::Currency::transfer(
                &caller,
                &recipient,
                amount,
                frame_support::traits::ExistenceRequirement::AllowDeath,
            )?;

            Ok(())
        })
    }

    /// Moves amount tokens from sender to recipient using the allowance mechanism,
    /// amount is then deducted from the caller’s allowance.
    pub fn transfer_from(
        caller: AccountIdOf<T, I>,
        sender: AccountIdOf<T, I>,
        recipient: AccountIdOf<T, I>,
        amount: BalanceOf<T, I>,
    ) -> DispatchResult {
        with_storage_layer(move || {
            <Approvals<T, I>>::mutate(sender.clone(), caller, |entry| {
                // Remove "value" from allowed, exit if underflow.

                let amount_as_allowance = T::Allowance::from(amount);

                if amount_as_allowance > *entry {
                    return Err(Error::<T, I>::SpendMoreThanAllowed);
                }

                let allowed = entry.sub(amount_as_allowance);

                // Update allowed value.
                *entry = allowed;

                Ok::<(), Error<T, I>>(())
            })?;

            T::Currency::transfer(
                &sender,
                &recipient,
                amount,
                frame_support::traits::ExistenceRequirement::AllowDeath,
            )?;

            Ok(())
        })
    }
}
