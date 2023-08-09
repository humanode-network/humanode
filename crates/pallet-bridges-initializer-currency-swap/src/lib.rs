//! A substrate pallet for bridges pot currency swap initialization logic.

// Either generate code at stadard mode, or `no_std`, based on the `std` feature presence.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    sp_runtime::{
        traits::{CheckedAdd, CheckedSub, Convert, Get, Zero},
        ArithmeticError, DispatchError,
    },
    storage::with_storage_layer,
    traits::{fungible, Currency, StorageVersion},
    weights::Weight,
};
pub use pallet::*;
use sp_std::cmp::Ordering;
pub use weights::*;

pub mod weights;

mod upgrade_init;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

/// The current bridges initializer version.
pub const CURRENT_BRIDGES_INITIALIZER_VERSION: u16 = 1;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, sp_runtime::traits::MaybeDisplay};
    use frame_system::pallet_prelude::*;
    use sp_std::fmt::Debug;

    use super::*;

    /// The Bridge Pot Currency Swap Initializer Pallet.
    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
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

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    /// The initializer version.
    #[pallet::storage]
    #[pallet::getter(fn initializer_version)]
    pub type InitializerVersion<T: Config> = StorageValue<_, u16, ValueQuery>;

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
            let is_balanced = Pallet::<T>::is_balanced().unwrap_or_default();

            if !is_balanced {
                match Pallet::<T>::initialize() {
                    Ok(_) => {}
                    Err(err) => panic!("error during bridges initialization: {err:?}",),
                }
            }

            <InitializerVersion<T>>::put(CURRENT_BRIDGES_INITIALIZER_VERSION);
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The currencies are not balanced.
        NotBalanced,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            upgrade_init::on_runtime_upgrade::<T>()
        }

        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
            upgrade_init::pre_upgrade::<T>()
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
            upgrade_init::post_upgrade::<T>(state)
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Verify if currencies are balanced.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::verify_balanced())]
        pub fn verify_balanced(_origin: OriginFor<T>) -> DispatchResult {
            if !Pallet::<T>::is_balanced()? {
                return Err(Error::<T>::NotBalanced.into());
            }

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Initialize bridges pot accounts.
    pub fn initialize() -> Result<Weight, DispatchError> {
        let mut weight = T::DbWeight::get().reads(0);

        with_storage_layer(move || {
            let evm_total_issuance = T::EvmCurrency::total_issuance();
            let evm_bridge_balance = T::EvmCurrency::total_balance(&T::EvmNativeBridgePot::get());
            weight += T::DbWeight::get().reads(2);

            let evm_swappable = evm_total_issuance
                .checked_sub(&evm_bridge_balance)
                .expect("evm_total_issuance is greater than evm_bridge_balance; qed.");

            let native_swap_reserved = T::BalanceConverterEvmToNative::convert(evm_swappable);
            let native_bridge_balance = native_swap_reserved
                .checked_add(&T::NativeCurrency::minimum_balance())
                .ok_or(ArithmeticError::Overflow)?;
            weight += T::DbWeight::get().reads(1);

            weight += Self::make_native_bridge_balance_be(native_bridge_balance)?;

            let native_total_issuance = T::NativeCurrency::total_issuance();
            weight += T::DbWeight::get().reads(1);

            let native_swappable = native_total_issuance
                .checked_sub(&native_bridge_balance)
                .expect("native_total_issuance is greater than native_bridge_balance; qed.");

            let evm_swap_reserved = T::BalanceConverterNativeToEvm::convert(native_swappable);
            let evm_bridge_balance = evm_swap_reserved
                .checked_add(&T::EvmCurrency::minimum_balance())
                .ok_or(ArithmeticError::Overflow)?;
            weight += T::DbWeight::get().reads(1);

            weight += Self::make_evm_bridge_balance_be(evm_bridge_balance)?;

            if !Self::is_balanced()? {
                return Err::<(), DispatchError>(Error::<T>::NotBalanced.into());
            }
            weight += T::DbWeight::get().reads(8);

            Ok(())
        })?;

        Ok(weight)
    }

    /// Make native bridge balance be provided amount value.
    ///
    /// The logic can change native swappable balance value.
    fn make_native_bridge_balance_be(
        amount: <T::NativeCurrency as Currency<T::AccountId>>::Balance,
    ) -> Result<Weight, DispatchError> {
        let current_native_bridge_balance =
            T::NativeCurrency::total_balance(&T::NativeEvmBridgePot::get());
        let mut weight = T::DbWeight::get().reads(1);

        if current_native_bridge_balance == Zero::zero() {
            let imbalance = T::NativeCurrency::withdraw(
                &T::NativeTreasuryPot::get(),
                amount,
                frame_support::traits::WithdrawReasons::TRANSFER,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;
            weight += T::DbWeight::get().writes(1);

            T::NativeCurrency::resolve_creating(&T::NativeEvmBridgePot::get(), imbalance);
            weight += T::DbWeight::get().writes(1);

            return Ok(weight);
        }

        match current_native_bridge_balance.cmp(&amount) {
            Ordering::Less => {
                let imbalance = T::NativeCurrency::withdraw(
                    &T::NativeTreasuryPot::get(),
                    amount
                        .checked_sub(&current_native_bridge_balance)
                        .expect("current_native_bridge_balance is less than amount; qed."),
                    frame_support::traits::WithdrawReasons::TRANSFER,
                    frame_support::traits::ExistenceRequirement::KeepAlive,
                )?;
                weight += T::DbWeight::get().writes(1);

                // We can safely ignore the result as overflow cann't be reached.
                // current_native_bridge_balance < amount. The resulted balance is equal to amount.
                let _ = T::NativeCurrency::resolve_into_existing(
                    &T::NativeEvmBridgePot::get(),
                    imbalance,
                );
                weight += T::DbWeight::get().writes(1);
            }
            Ordering::Greater => {
                let imbalance = T::NativeCurrency::withdraw(
                    &T::NativeEvmBridgePot::get(),
                    current_native_bridge_balance
                        .checked_sub(&amount)
                        .expect("current_native_bridge_balance is greater than amount; qed."),
                    frame_support::traits::WithdrawReasons::TRANSFER,
                    frame_support::traits::ExistenceRequirement::KeepAlive,
                )?;
                weight += T::DbWeight::get().writes(1);

                // We can safely ignore the result as overflow cann't be reached.
                // current_native_bridge_balance + current_native_treasury < total_issuance.
                // So, imbalance + current_native_treasury < total_issuance.
                let _ = T::NativeCurrency::resolve_into_existing(
                    &T::NativeTreasuryPot::get(),
                    imbalance,
                );
                weight += T::DbWeight::get().writes(1);
            }
            Ordering::Equal => {}
        }

        Ok(weight)
    }

    /// Make evm bridge balance be provided amount value.
    ///
    /// The logic shouldn't change evm swappable balance value.
    fn make_evm_bridge_balance_be(
        amount: <T::EvmCurrency as Currency<T::EvmAccountId>>::Balance,
    ) -> Result<Weight, DispatchError> {
        let current_evm_bridge_balance =
            T::EvmCurrency::total_balance(&T::EvmNativeBridgePot::get());
        let mut weight = T::DbWeight::get().reads(1);

        if current_evm_bridge_balance == Zero::zero() {
            let imbalance = T::EvmCurrency::issue(amount);
            weight += T::DbWeight::get().writes(1);

            T::EvmCurrency::resolve_creating(&T::EvmNativeBridgePot::get(), imbalance);
            weight += T::DbWeight::get().writes(1);

            return Ok(weight);
        }

        match current_evm_bridge_balance.cmp(&amount) {
            Ordering::Less => {
                let imbalance = T::EvmCurrency::issue(
                    amount
                        .checked_sub(&current_evm_bridge_balance)
                        .expect("current_evm_bridge_balance is less than amount; qed."),
                );
                weight += T::DbWeight::get().writes(1);

                // We can safely ignore the result as overflow cann't be reached.
                // current_evm_bridge_balance < amount. The resulted balance is equal to amount.
                let _ =
                    T::EvmCurrency::resolve_into_existing(&T::EvmNativeBridgePot::get(), imbalance);
                weight += T::DbWeight::get().writes(1);
            }
            Ordering::Greater => {
                let imbalance = T::EvmCurrency::burn(
                    current_evm_bridge_balance
                        .checked_sub(&amount)
                        .expect("current_evm_bridge_balance is greater than amount; qed."),
                );
                weight += T::DbWeight::get().writes(1);

                // We can safely ignore the result as underflow cann't be reached.
                // current_evm_bridge_balance > amount => imbalance < current_evm_bridge_balance.
                let _ = T::EvmCurrency::settle(
                    &T::EvmNativeBridgePot::get(),
                    imbalance,
                    frame_support::traits::WithdrawReasons::RESERVE,
                    frame_support::traits::ExistenceRequirement::KeepAlive,
                );
                weight += T::DbWeight::get().writes(1);
            }
            Ordering::Equal => {}
        }
        Ok(weight)
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
