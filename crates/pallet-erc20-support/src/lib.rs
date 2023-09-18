//! A substrate pallet that exposes currency instance using the ERC20 interface standard..

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    sp_runtime::{traits::CheckedSub, DispatchResult},
    storage::with_storage_layer,
    traits::{Currency, StorageVersion},
};
pub use pallet::*;

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
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

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

    use super::*;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::generate_store(pub(super) trait Store)]
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
        BalanceOf<T, I>,
        ValueQuery,
    >;

    /// Possible errors.
    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Spender can't transfer tokens more than allowed.
        SpendMoreThanAllowed,
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
    pub fn allowance(owner: &AccountIdOf<T, I>, spender: &AccountIdOf<T, I>) -> BalanceOf<T, I> {
        <Approvals<T, I>>::get(owner, spender)
    }

    /// Sets amount as the allowance of spender over the caller’s tokens.
    pub fn approve(owner: AccountIdOf<T, I>, spender: AccountIdOf<T, I>, amount: BalanceOf<T, I>) {
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
                let allowed = entry
                    .checked_sub(&amount)
                    .ok_or(Error::<T, I>::SpendMoreThanAllowed)?;

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
