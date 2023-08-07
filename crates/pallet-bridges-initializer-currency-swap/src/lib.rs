//! A substrate pallet for bridge pot currency swap initialization implementation.

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

pub trait BalanceMaker<AccountId, C: Currency<AccountId>> {
    const IS_SWAPPABLE_CHANGED: bool;

    fn make_balance(amount: C::Balance) -> Result<(), DispatchError>;
}

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
        type EvmAccountId: Parameter
            + Member
            + MaybeSerializeDeserialize
            + Debug
            + MaybeDisplay
            + Ord
            + MaxEncodedLen;

        type NativeCurrency: Currency<Self::AccountId>
            + fungible::Inspect<
                Self::AccountId,
                Balance = <Self::NativeCurrency as Currency<Self::AccountId>>::Balance,
            >;

        type EvmCurrency: Currency<Self::EvmAccountId>
            + fungible::Inspect<
                Self::EvmAccountId,
                Balance = <Self::EvmCurrency as Currency<Self::EvmAccountId>>::Balance,
            >;

        type BalanceConverterNativeToEvm: Convert<
            <Self::NativeCurrency as Currency<Self::AccountId>>::Balance,
            <Self::EvmCurrency as Currency<Self::EvmAccountId>>::Balance,
        >;

        type BalanceConverterEvmToNative: Convert<
            <Self::EvmCurrency as Currency<Self::EvmAccountId>>::Balance,
            <Self::NativeCurrency as Currency<Self::AccountId>>::Balance,
        >;

        type NativeEvmBridgePot: Get<Self::AccountId>;
        type NativeBridgeBalanceMaker: BalanceMaker<Self::AccountId, Self::NativeCurrency>;

        type EvmNativeBridgePot: Get<Self::EvmAccountId>;
        type EvmBridgeBalanceMaker: BalanceMaker<Self::EvmAccountId, Self::EvmCurrency>;
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
            match Pallet::<T>::init() {
                Ok(_) => {}
                Err(err) => panic!("error during bridges initialization: {err:?}",),
            }
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn init() -> Result<(), DispatchError> {
        assert!(
            !T::EvmBridgeBalanceMaker::IS_SWAPPABLE_CHANGED,
            "not supported"
        );

        let evm_total_issuance = T::EvmCurrency::total_issuance();
        let evm_swappable = evm_total_issuance;

        let native_swap_reserved = T::BalanceConverterEvmToNative::convert(evm_swappable);
        let native_bridge_balance = native_swap_reserved
            .checked_add(&T::NativeCurrency::minimum_balance())
            .ok_or(ArithmeticError::Overflow)?;

        T::NativeBridgeBalanceMaker::make_balance(native_bridge_balance)?;

        let native_total_issuance = T::NativeCurrency::total_issuance();
        let native_swappable = native_total_issuance
            .checked_sub(&native_bridge_balance)
            .ok_or(ArithmeticError::Underflow)?;

        let evm_swap_reserved = T::BalanceConverterNativeToEvm::convert(native_swappable);
        let evm_bridge_balance = evm_swap_reserved
            .checked_add(&T::EvmCurrency::minimum_balance())
            .ok_or(ArithmeticError::Overflow)?;

        T::EvmBridgeBalanceMaker::make_balance(evm_bridge_balance)?;

        assert!(Self::verify_balanced()?, "not balanced");

        Ok(())
    }

    fn verify_balanced() -> Result<bool, ArithmeticError> {
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

/// Swappable balance.
fn swappable_balance<AccountId, C: Currency<AccountId>, B: Get<AccountId>>(
) -> Result<C::Balance, ArithmeticError> {
    let total = C::total_issuance();
    let bridge = C::total_balance(&B::get());

    let swappable = total
        .checked_sub(&bridge)
        .ok_or(ArithmeticError::Underflow)?;

    Ok(swappable)
}

/// Swap reserved balance.
fn swap_reserved_balance<AccountId, C: Currency<AccountId>, B: Get<AccountId>>(
) -> Result<C::Balance, ArithmeticError> {
    let bridge = C::total_balance(&B::get());
    let ed = C::minimum_balance();

    let reserved = bridge.checked_sub(&ed).ok_or(ArithmeticError::Underflow)?;

    Ok(reserved)
}
