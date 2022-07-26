//! The vesting pallet.

#![cfg_attr(not(feature = "std"), no_std)]

mod vesting_info;

use codec::MaxEncodedLen;
use frame_support::{
    pallet_prelude::*,
    storage::bounded_vec::BoundedVec,
    traits::{Currency, LockIdentifier, LockableCurrency, StorageVersion, WithdrawReasons},
};
pub use pallet::*;
use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, Zero};
use vesting_info::VestingInfo;
use vesting_schedule::VestingSchedule;

/// Balance type alias.
type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
/// Full VestingInfo type.
type FullVestingInfo<T> = VestingInfo<BalanceOf<T>, <T as Config>::Moment>;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

/// Provides the capability to get current moment.
pub trait CurrentMoment<Moment> {
    /// Return current moment.
    fn now() -> Moment;
}

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The currency to operate with.
        type Currency: LockableCurrency<Self::AccountId>;

        /// The vesting schedule to operate with.
        type VestingSchedule: VestingSchedule<Self::AccountId>;

        /// Type used for expressing moment.
        type Moment: Parameter
            + Default
            + AtLeast32BitUnsigned
            + Copy
            + MaybeSerializeDeserialize
            + MaxEncodedLen;

        /// The getter for the current moment.
        type CurrentMoment: CurrentMoment<Self::Moment>;

        /// Maximum number of vesting schedules an account may have at a given moment.
        type MaxVestingSchedules: Get<u32>;

        /// An lock identifier for a lockable currency to be used in vesting.
        type VestingLockId: Get<LockIdentifier>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// Information regarding the vesting of a given account.
    #[pallet::storage]
    #[pallet::getter(fn vesting)]
    pub type Vesting<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<FullVestingInfo<T>, T::MaxVestingSchedules>,
    >;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// The list of vesting to use.
        pub vesting: Vec<(T::AccountId, BalanceOf<T>, T::Moment)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig {
                vesting: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            for &(ref who, locked, start) in self.vesting.iter() {
                let balance = T::Currency::free_balance(who);
                assert!(
                    !balance.is_zero(),
                    "Currencies must be init'd before vesting"
                );

                // TODO. Vesting validate

                let vesting_info = VestingInfo { locked, start };

                Vesting::<T>::try_append(who, vesting_info)
                    .expect("Too many vesting schedules at genesis.");

                let reasons = WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE;
                T::Currency::set_lock(T::VestingLockId::get(), who, locked, reasons);
            }
        }
    }
}
