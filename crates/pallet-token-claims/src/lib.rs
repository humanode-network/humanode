//! Token claims.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, StorageVersion};

mod types;
mod weights;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

/// The currency from a given config.
type CurrencyOf<T> = <T as Config>::Currency;
/// The balance from a given config.
type BalanceOf<T> = <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use primitives_ethereum::EthereumAddress;

    use super::*;
    use crate::{types::ClaimInfo, weights::WeightInfo};

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Currency to claim.
        type Currency: Currency<<Self as frame_system::Config>::AccountId>;

        /// Vesting schedule configuration type.
        type VestingSchedule: Member + Parameter;

        /// The weight informtation provider type.
        type WeightInfo: WeightInfo;
    }

    /// The public key of the robonode.
    #[pallet::storage]
    #[pallet::getter(fn claims)]
    pub type Claims<T> = StorageMap<
        _,
        Twox64Concat,
        EthereumAddress,
        ClaimInfo<BalanceOf<T>, <T as Config>::VestingSchedule>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {}
}
pub use pallet::*;
