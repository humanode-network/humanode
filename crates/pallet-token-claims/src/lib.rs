//! Token claims.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, StorageVersion};

pub use self::pallet::*;

pub mod traits;
mod types;
mod weights;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

/// The currency from a given config.
type CurrencyOf<T> = <T as Config>::Currency;
/// The balance from a given config.
type BalanceOf<T> = <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
/// The claim info from a given config.
type ClaimInfoOf<T> = types::ClaimInfo<BalanceOf<T>, <T as Config>::VestingSchedule>;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use primitives_ethereum::{EcdsaSignature, EthereumAddress};

    use super::*;
    use crate::{
        traits::{PreconstructedMessageVerifier, VestingInterface},
        types::{ClaimInfo, EthereumSignatureMessageParams},
        weights::WeightInfo,
    };

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
        type VestingSchedule: Member + Parameter + MaxEncodedLen;

        /// Interface into the vesting implementation.
        type VestingInterface: VestingInterface<
            AccountId = Self::AccountId,
            Balance = BalanceOf<Self>,
            Schedule = Self::VestingSchedule,
        >;

        /// The ethereum signature verifier for the claim requests.
        type EthereumSignatureVerifier: PreconstructedMessageVerifier<
            MessageParams = EthereumSignatureMessageParams<Self::AccountId>,
        >;

        /// The weight informtation provider type.
        type WeightInfo: WeightInfo;
    }

    /// The public key of the robonode.
    #[pallet::storage]
    #[pallet::getter(fn claims)]
    pub type Claims<T> = StorageMap<_, Twox64Concat, EthereumAddress, ClaimInfoOf<T>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Tokens were claimed.
        TokensClaimed {
            /// Who claimed the tokens.
            who: T::AccountId,
            /// The ethereum address used for token claiming.
            ethereum_address: EthereumAddress,
            /// The balance that was claimed.
            balance: BalanceOf<T>,
            /// The vesting schedule.
            vesting: Option<T::VestingSchedule>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The signature was invalid.
        InvalidSignature,
        /// No claim was found.
        NoClaim,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Claim the tokens.
        #[pallet::weight(T::WeightInfo::claim())]
        pub fn claim(
            origin: OriginFor<T>,
            ethereum_address: EthereumAddress,
            ethereum_signature: EcdsaSignature,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let message_params = EthereumSignatureMessageParams {
                account_id: who.clone(),
                ethereum_address,
            };

            if <T as Config>::EthereumSignatureVerifier::verify(
                message_params,
                &ethereum_address,
                ethereum_signature,
            ) {
                return Err(Error::<T>::InvalidSignature.into());
            }

            Self::process_claim(who, ethereum_address)
        }
    }

    impl<T: Config> Pallet<T> {
        fn process_claim(who: T::AccountId, ethereum_address: EthereumAddress) -> DispatchResult {
            let ClaimInfo { balance, vesting } =
                <Claims<T>>::take(ethereum_address).ok_or(<Error<T>>::NoClaim)?;

            T::Currency::deposit_creating(&who, balance);

            if let Some(ref vesting) = vesting {
                T::VestingInterface::lock_under_vesting(&who, balance, vesting.clone())?;
            }

            Self::deposit_event(Event::TokensClaimed {
                who,
                ethereum_address,
                balance,
                vesting,
            });

            Ok(())
        }
    }
}
