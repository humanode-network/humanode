//! A substrate pallet for bridges pot currency swap initialization logic.

// Either generate code at stadard mode, or `no_std`, based on the `std` feature presence.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    sp_runtime::{
        traits::{CheckedAdd, CheckedSub, Convert, Get},
        ArithmeticError, DispatchError,
    },
    traits::{fungible, Currency},
};
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, sp_runtime::traits::MaybeDisplay};
    use sp_std::fmt::Debug;

    use super::*;

    /// The Bridge Pot Currency Swap Pallet.
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configuration trait of this pallet.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The evm user account identifier type.
        type EvmAccountId: Parameter
            + Member
            + MaybeSerializeDeserialize
            + Debug
            + MaybeDisplay
            + Ord
            + MaxEncodedLen;

        /// The interface into native currency implementation.
        type NativeCurrency: Currency<Self::AccountId>
            + fungible::Inspect<
                Self::AccountId,
                Balance = <Self::NativeCurrency as Currency<Self::AccountId>>::Balance,
            >;

        /// The interface into evm currency implementation.
        type EvmCurrency: Currency<Self::EvmAccountId>
            + fungible::Inspect<
                Self::EvmAccountId,
                Balance = <Self::EvmCurrency as Currency<Self::EvmAccountId>>::Balance,
            >;

        /// The converter to determine how the balance amount should be converted from
        /// native currency to evm currency.
        type BalanceConverterNativeToEvm: Convert<
            <Self::NativeCurrency as Currency<Self::AccountId>>::Balance,
            <Self::EvmCurrency as Currency<Self::EvmAccountId>>::Balance,
        >;

        /// The converter to determine how the balance amount should be converted from
        /// evm currency to native currency.
        type BalanceConverterEvmToNative: Convert<
            <Self::EvmCurrency as Currency<Self::EvmAccountId>>::Balance,
            <Self::NativeCurrency as Currency<Self::AccountId>>::Balance,
        >;

        /// The native-evm bridge pot account.
        type NativeEvmBridgePot: Get<Self::AccountId>;

        /// The native treasury pot account.
        type NativeTreasuryPot: Get<Self::AccountId>;

        /// The evm-native bridge pot account.
        type EvmNativeBridgePot: Get<Self::EvmAccountId>;
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config>(PhantomData<T>);

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }

    // The build of genesis for the pallet.
    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            match Pallet::<T>::initialize() {
                Ok(_) => {}
                Err(err) => panic!("error during bridges initialization: {err:?}",),
            }
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The currencies are not balanced.
        NotBalanced,
    }
}

impl<T: Config> Pallet<T> {
    /// Initialize bridges pot accounts.
    pub fn initialize() -> Result<(), DispatchError> {
        let evm_total_issuance = T::EvmCurrency::total_issuance();
        let evm_swappable = evm_total_issuance;

        let native_swap_reserved = T::BalanceConverterEvmToNative::convert(evm_swappable);
        let native_bridge_balance = native_swap_reserved
            .checked_add(&T::NativeCurrency::minimum_balance())
            .ok_or(ArithmeticError::Overflow)?;

        let imbalance = T::NativeCurrency::withdraw(
            &T::NativeTreasuryPot::get(),
            native_bridge_balance,
            frame_support::traits::WithdrawReasons::TRANSFER,
            frame_support::traits::ExistenceRequirement::KeepAlive,
        )?;
        T::NativeCurrency::resolve_creating(&T::NativeEvmBridgePot::get(), imbalance);

        let native_total_issuance = T::NativeCurrency::total_issuance();
        let native_swappable = native_total_issuance
            .checked_sub(&native_bridge_balance)
            .ok_or(ArithmeticError::Underflow)?;

        let evm_swap_reserved = T::BalanceConverterNativeToEvm::convert(native_swappable);
        let evm_bridge_balance = evm_swap_reserved
            .checked_add(&T::EvmCurrency::minimum_balance())
            .ok_or(ArithmeticError::Overflow)?;

        let imbalance = T::EvmCurrency::issue(evm_bridge_balance);
        T::EvmCurrency::resolve_creating(&T::EvmNativeBridgePot::get(), imbalance);

        if !Self::is_balanced()? {
            return Err(Error::<T>::NotBalanced.into());
        }

        Ok(())
    }

    /// Verify currencies balanced requirements.
    fn is_balanced() -> Result<bool, ArithmeticError> {
        let is_balanced_native_evm = swap_reserved_balance::<
            T::AccountId,
            T::NativeCurrency,
            T::NativeEvmBridgePot,
        >()? == T::BalanceConverterEvmToNative::convert(
            swappable_balance::<T::EvmAccountId, T::EvmCurrency, T::EvmNativeBridgePot>()?,
        );

        let is_balanced_evm_native =
            T::BalanceConverterNativeToEvm::convert(swap_reserved_balance::<
                T::AccountId,
                T::NativeCurrency,
                T::NativeEvmBridgePot,
            >()?)
                == swappable_balance::<T::EvmAccountId, T::EvmCurrency, T::EvmNativeBridgePot>()?;

        Ok(is_balanced_native_evm && is_balanced_evm_native)
    }
}

/// A helper function to calculate swappable balance.
fn swappable_balance<AccountId, C: Currency<AccountId>, B: Get<AccountId>>(
) -> Result<C::Balance, ArithmeticError> {
    let total = C::total_issuance();
    let bridge = C::total_balance(&B::get());

    let swappable = total
        .checked_sub(&bridge)
        .ok_or(ArithmeticError::Underflow)?;

    Ok(swappable)
}

/// A helper function to calculate swap reserved balance.
fn swap_reserved_balance<AccountId, C: Currency<AccountId>, B: Get<AccountId>>(
) -> Result<C::Balance, ArithmeticError> {
    let bridge = C::total_balance(&B::get());
    let ed = C::minimum_balance();

    let reserved = bridge.checked_sub(&ed).ok_or(ArithmeticError::Underflow)?;

    Ok(reserved)
}
