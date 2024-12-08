//! Token claims.
//!
//! # Security
//!
//! This pallet requires adding [`CheckTokenClaim`] to the tuple of signed extension checkers
//! at runtime to be utilized safely, otherwise it exposes a flooding vulnerability.
//! There is no way to ensure this would be automatically picked up by the runtime, so double-check
//! it at integration!

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, StorageVersion};
pub use weights::*;

pub use self::pallet::*;
pub use self::signed_ext::*;

mod signed_ext;
pub mod traits;
pub mod types;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

/// The currency from a given config.
type CurrencyOf<T> = <T as Config>::Currency;
/// The balance from a given config.
type BalanceOf<T> = <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
/// The claim info from a given config.
pub type ClaimInfoOf<T> = types::ClaimInfo<BalanceOf<T>, <T as Config>::VestingSchedule>;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::sp_runtime::traits::{CheckedAdd, Zero};
    use frame_support::{
        pallet_prelude::{ValueQuery, *},
        sp_runtime::traits::Saturating,
        sp_std::prelude::*,
        storage::with_storage_layer,
        traits::{ExistenceRequirement, WithdrawReasons},
    };
    use frame_system::pallet_prelude::*;
    use primitives_ethereum::{EcdsaSignature, EthereumAddress};

    use super::*;
    use crate::{
        traits::{verify_ethereum_signature, EthereumSignatureVerifier, VestingInterface},
        types::{ClaimInfo, EthereumSignatureMessageParams},
        weights::WeightInfo,
    };

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Currency to claim.
        type Currency: Currency<<Self as frame_system::Config>::AccountId>;

        /// The ID for the pot account to use.
        #[pallet::constant]
        type PotAccountId: Get<<Self as frame_system::Config>::AccountId>;

        /// Vesting schedule configuration type.
        type VestingSchedule: Member + Parameter + MaxEncodedLen + MaybeSerializeDeserialize;

        /// Interface into the vesting implementation.
        type VestingInterface: VestingInterface<
            AccountId = Self::AccountId,
            Balance = BalanceOf<Self>,
            Schedule = Self::VestingSchedule,
        >;

        /// The ethereum signature verifier for the claim requests.
        type EthereumSignatureVerifier: EthereumSignatureVerifier<
            MessageParams = EthereumSignatureMessageParams<Self::AccountId>,
        >;

        /// The weight informtation provider type.
        type WeightInfo: WeightInfo;
    }

    /// The claims that are available in the system.
    #[pallet::storage]
    #[pallet::getter(fn claims)]
    pub type Claims<T> = StorageMap<_, Twox64Concat, EthereumAddress, ClaimInfoOf<T>, OptionQuery>;

    /// The total amount of claimable balance in the pot.
    #[pallet::storage]
    #[pallet::getter(fn total_claimable)]
    pub type TotalClaimable<T> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// The claims to initialize at genesis.
        pub claims: Vec<(EthereumAddress, ClaimInfoOf<T>)>,
        /// The total claimable balance.
        ///
        /// If provided, must be equal to the sum of all claims balances.
        /// This is useful for double-checking the expected sum during the genesis construction.
        pub total_claimable: Option<BalanceOf<T>>,
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            let mut total_claimable_balance: BalanceOf<T> = Zero::zero();

            for (eth_address, info) in &self.claims {
                if Claims::<T>::contains_key(eth_address) {
                    panic!("conflicting claim found in genesis for address {eth_address}");
                }

                Claims::<T>::insert(eth_address, info.clone());
                total_claimable_balance =
                    total_claimable_balance.checked_add(&info.balance).unwrap();
            }

            // Ensure that our pot account has exactly the right balance.
            let expected_pot_balance = <CurrencyOf<T>>::minimum_balance()
                .checked_add(&total_claimable_balance)
                .unwrap();
            let pot_account_id = T::PotAccountId::get();
            let actual_pot_balance = <CurrencyOf<T>>::free_balance(&pot_account_id);
            if actual_pot_balance != expected_pot_balance {
                panic!(
                    "invalid balance in the token claims pot account: got {actual_pot_balance:?}, expected {expected_pot_balance:?}"
                );
            }

            // Check that the total claimable balance we computed matched the one declared in the
            // genesis configuration.
            if let Some(expected_total_claimable_balance) = self.total_claimable {
                if expected_total_claimable_balance != total_claimable_balance {
                    panic!(
                        "computed total claimable balance ({total_claimable_balance:?}) is different from the one specified at the genesis config ({expected_total_claimable_balance:?})"
                    );
                }
            }

            // Initialize the total claimable balance.
            <Pallet<T>>::update_total_claimable_balance();
        }
    }

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
            vesting: T::VestingSchedule,
        },
        /// Claim were added.
        ClaimAdded {
            /// The ethereum address used for token claiming.
            ethereum_address: EthereumAddress,
            /// The claim info that was claimed.
            claim: ClaimInfoOf<T>,
        },
        /// Claim were removed.
        ClaimRemoved {
            /// The ethereum address that was removed.
            ethereum_address: EthereumAddress,
            /// The claim info that was removed.
            claim: ClaimInfoOf<T>,
        },
        /// Claim were changed.
        ClaimChanged {
            /// The ethereum address used for token claiming.
            ethereum_address: EthereumAddress,
            /// An old claim info.
            old_claim: ClaimInfoOf<T>,
            /// A new claim info.
            new_claim: ClaimInfoOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The signature was invalid.
        InvalidSignature,
        /// No claim was found.
        NoClaim,
        /// Conflicting ethereum address.
        ConflictingEthereumAddress,
        /// Pot balance is too high.
        ClaimsPotOverflow,
        /// Pot balance is too low.
        ClaimsPotUnderflow,
        /// Funds provider balance is too high.
        FundsProviderOverflow,
        /// Funds provider balance is too low.
        FundsProviderUnderflow,
        /// Funds consumer balance is too high.
        FundsConsumerOverflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Claim the tokens.
        #[pallet::call_index(0)]
        #[pallet::weight((T::WeightInfo::claim(), Pays::No))]
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

            if !verify_ethereum_signature::<<T as Config>::EthereumSignatureVerifier>(
                &ethereum_signature,
                &message_params,
                &ethereum_address,
            ) {
                return Err(Error::<T>::InvalidSignature.into());
            }

            Self::process_claim(who, ethereum_address)
        }

        /// Add a new claim.
        #[pallet::call_index(1)]
        #[pallet::weight((T::WeightInfo::add_claim(), Pays::No))]
        pub fn add_claim(
            origin: OriginFor<T>,
            ethereum_address: EthereumAddress,
            claim_info: ClaimInfoOf<T>,
            funds_provider: T::AccountId,
        ) -> DispatchResult {
            ensure_root(origin)?;

            if <Claims<T>>::contains_key(ethereum_address) {
                return Err(Error::<T>::ConflictingEthereumAddress.into());
            }

            with_storage_layer(move || {
                let funds = <CurrencyOf<T>>::withdraw(
                    &funds_provider,
                    claim_info.balance,
                    WithdrawReasons::TRANSFER,
                    ExistenceRequirement::KeepAlive,
                )
                .map_err(|_| Error::<T>::FundsProviderUnderflow)?;

                <CurrencyOf<T>>::resolve_into_existing(&T::PotAccountId::get(), funds)
                    .map_err(|_| Error::<T>::ClaimsPotOverflow)?;

                Claims::<T>::insert(ethereum_address, claim_info.clone());

                <Pallet<T>>::update_total_claimable_balance();

                Self::deposit_event(Event::ClaimAdded {
                    ethereum_address,
                    claim: claim_info,
                });

                Ok(())
            })
        }

        /// Remove an existing claim.
        #[pallet::call_index(2)]
        #[pallet::weight((T::WeightInfo::remove_claim(), Pays::No))]
        pub fn remove_claim(
            origin: OriginFor<T>,
            ethereum_address: EthereumAddress,
            funds_consumer: T::AccountId,
        ) -> DispatchResult {
            ensure_root(origin)?;

            with_storage_layer(move || {
                let claim_info = <Claims<T>>::take(ethereum_address).ok_or(<Error<T>>::NoClaim)?;

                let funds = <CurrencyOf<T>>::withdraw(
                    &T::PotAccountId::get(),
                    claim_info.balance,
                    WithdrawReasons::TRANSFER,
                    ExistenceRequirement::KeepAlive,
                )
                .map_err(|_| Error::<T>::ClaimsPotUnderflow)?;

                <CurrencyOf<T>>::resolve_into_existing(&funds_consumer, funds)
                    .map_err(|_| Error::<T>::FundsConsumerOverflow)?;

                <Pallet<T>>::update_total_claimable_balance();

                Self::deposit_event(Event::ClaimRemoved {
                    ethereum_address,
                    claim: claim_info,
                });

                Ok(())
            })
        }

        /// Change an existing claim.
        #[pallet::call_index(3)]
        #[pallet::weight((T::WeightInfo::change_claim(), Pays::No))]
        pub fn change_claim(
            origin: OriginFor<T>,
            ethereum_address: EthereumAddress,
            claim_info: ClaimInfoOf<T>,
            funds_provider: T::AccountId,
        ) -> DispatchResult {
            ensure_root(origin)?;

            with_storage_layer(move || {
                let old_claim = <Claims<T>>::take(ethereum_address).ok_or(<Error<T>>::NoClaim)?;

                use frame_support::sp_runtime::traits::{CheckedSub, Zero};

                if let Some(increase) = claim_info.balance.checked_sub(&old_claim.balance) {
                    if !increase.is_zero() {
                        let funds = <CurrencyOf<T>>::withdraw(
                            &funds_provider,
                            increase,
                            WithdrawReasons::TRANSFER,
                            ExistenceRequirement::KeepAlive,
                        )
                        .map_err(|_| Error::<T>::FundsProviderUnderflow)?;

                        <CurrencyOf<T>>::resolve_into_existing(&T::PotAccountId::get(), funds)
                            .map_err(|_| Error::<T>::ClaimsPotOverflow)?;
                    }
                } else if let Some(decrease) = old_claim.balance.checked_sub(&claim_info.balance) {
                    let funds = <CurrencyOf<T>>::withdraw(
                        &T::PotAccountId::get(),
                        decrease,
                        WithdrawReasons::TRANSFER,
                        ExistenceRequirement::KeepAlive,
                    )
                    .map_err(|_| Error::<T>::ClaimsPotUnderflow)?;

                    <CurrencyOf<T>>::resolve_into_existing(&funds_provider, funds)
                        .map_err(|_| Error::<T>::FundsProviderOverflow)?;
                }

                Claims::<T>::insert(ethereum_address, claim_info.clone());

                <Pallet<T>>::update_total_claimable_balance();

                Self::deposit_event(Event::ClaimChanged {
                    ethereum_address,
                    old_claim,
                    new_claim: claim_info,
                });

                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        fn process_claim(who: T::AccountId, ethereum_address: EthereumAddress) -> DispatchResult {
            with_storage_layer(move || {
                let ClaimInfo { balance, vesting } =
                    <Claims<T>>::take(ethereum_address).ok_or(<Error<T>>::NoClaim)?;

                let funds = <CurrencyOf<T>>::withdraw(
                    &T::PotAccountId::get(),
                    balance,
                    WithdrawReasons::TRANSFER,
                    ExistenceRequirement::KeepAlive,
                )?;
                <CurrencyOf<T>>::resolve_creating(&who, funds);

                T::VestingInterface::lock_under_vesting(&who, balance, vesting.clone())?;

                Self::update_total_claimable_balance();

                Self::deposit_event(Event::TokensClaimed {
                    who,
                    ethereum_address,
                    balance,
                    vesting,
                });

                Ok(())
            })
        }

        fn update_total_claimable_balance() {
            <TotalClaimable<T>>::set(
                <CurrencyOf<T>>::free_balance(&T::PotAccountId::get())
                    .saturating_sub(<CurrencyOf<T>>::minimum_balance()),
            );
        }
    }
}
