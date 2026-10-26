//! A pallet that holds the fee parameters adjustable at the state level.
//!
//! The pallet stores the multipliers intended to be applied on top of the base transaction fees,
//! and allows the root origin to change them.
//! It also provides the adapters that apply the multipliers to the fee computation.

#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;

use fp_evm::FeeCalculator;
use frame_support::{
    traits::{Get, StorageVersion},
    weights::{Weight, WeightToFee},
};
use sp_core::U256;
use sp_runtime::{FixedPointNumber, FixedPointOperand, FixedU128};

pub use self::pallet::*;
pub use self::weights::*;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
pub mod migrations;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod weights;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The weight information provider type.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// The default native fee multiplier.
    ///
    /// The identity multiplier, keeps the base fees as is.
    #[pallet::type_value]
    pub fn DefaultNativeFeeMultiplier() -> FixedU128 {
        FixedU128::from_u32(1)
    }

    /// The default EVM fee multiplier.
    ///
    /// Double the default native fee multiplier: the native path is the more efficient way
    /// to transact, so the EVM path is priced higher to push the users towards the native one.
    #[pallet::type_value]
    pub fn DefaultEvmFeeMultiplier() -> FixedU128 {
        FixedU128::from_u32(2)
    }

    /// The multiplier to apply on top of the native transaction fees.
    #[pallet::storage]
    #[pallet::getter(fn native_fee_multiplier)]
    pub type NativeFeeMultiplier<T: Config> =
        StorageValue<_, FixedU128, ValueQuery, DefaultNativeFeeMultiplier>;

    /// The multiplier to apply on top of the EVM transaction fees.
    #[pallet::storage]
    #[pallet::getter(fn evm_fee_multiplier)]
    pub type EvmFeeMultiplier<T: Config> =
        StorageValue<_, FixedU128, ValueQuery, DefaultEvmFeeMultiplier>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        /// The multiplier to apply on top of the native transaction fees.
        pub native_fee_multiplier: FixedU128,

        /// The multiplier to apply on top of the EVM transaction fees.
        pub evm_fee_multiplier: FixedU128,
    }

    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                native_fee_multiplier: DefaultNativeFeeMultiplier::get(),
                evm_fee_multiplier: DefaultEvmFeeMultiplier::get(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            <NativeFeeMultiplier<T>>::put(self.native_fee_multiplier);
            <EvmFeeMultiplier<T>>::put(self.evm_fee_multiplier);
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The native fee multiplier has been updated.
        NativeFeeMultiplierUpdated {
            /// The new multiplier.
            multiplier: FixedU128,
        },

        /// The EVM fee multiplier has been updated.
        EvmFeeMultiplierUpdated {
            /// The new multiplier.
            multiplier: FixedU128,
        },
    }

    #[pallet::call(weight(T::WeightInfo))]
    impl<T: Config> Pallet<T> {
        /// Sets the multiplier to apply on top of the native transaction fees.
        #[pallet::call_index(0)]
        pub fn set_native_fee_multiplier(
            origin: OriginFor<T>,
            multiplier: FixedU128,
        ) -> DispatchResult {
            ensure_root(origin)?;

            <NativeFeeMultiplier<T>>::put(multiplier);
            Self::deposit_event(Event::NativeFeeMultiplierUpdated { multiplier });

            Ok(())
        }

        /// Sets the multiplier to apply on top of the EVM transaction fees.
        #[pallet::call_index(1)]
        pub fn set_evm_fee_multiplier(
            origin: OriginFor<T>,
            multiplier: FixedU128,
        ) -> DispatchResult {
            ensure_root(origin)?;

            <EvmFeeMultiplier<T>>::put(multiplier);
            Self::deposit_event(Event::EvmFeeMultiplierUpdated { multiplier });

            Ok(())
        }
    }
}

/// The native weight to fee conversion: the `Baseline` conversion scaled by the native fee
/// multiplier.
pub struct NativeWeightToFee<T, Baseline>(PhantomData<(T, Baseline)>);

impl<T, Baseline> WeightToFee for NativeWeightToFee<T, Baseline>
where
    T: Config,
    Baseline: WeightToFee,
    Baseline::Balance: FixedPointOperand,
{
    type Balance = Baseline::Balance;

    fn weight_to_fee(weight: &Weight) -> Self::Balance {
        let baseline_fee = Baseline::weight_to_fee(weight);
        <NativeFeeMultiplier<T>>::get().saturating_mul_int(baseline_fee)
    }
}

/// The EVM fee calculator: the `BaselineFeePerGas` scaled by the EVM fee multiplier.
pub struct EvmFeeCalculator<T, BaselineFeePerGas>(PhantomData<(T, BaselineFeePerGas)>);

impl<T, BaselineFeePerGas> FeeCalculator for EvmFeeCalculator<T, BaselineFeePerGas>
where
    T: Config,
    BaselineFeePerGas: Get<u128>,
{
    fn min_gas_price() -> (U256, Weight) {
        let fee_per_gas = <EvmFeeMultiplier<T>>::get().saturating_mul_int(BaselineFeePerGas::get());
        (fee_per_gas.into(), T::DbWeight::get().reads(1))
    }
}
