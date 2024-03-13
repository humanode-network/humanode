//! A substrate pallet that provides logic to map Substrate and EVM accounts.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{inherent::Vec, pallet_prelude::*};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use primitives_ethereum::{EcdsaSignature, EthereumAddress};
pub use weights::*;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// The verifier for the ethereum signature.
pub trait SignedClaimVerifier {
    /// The type of the native account on the chain.
    type AccountId;

    /// Verify the provided `signature` against a message declaring a claim of the provided
    /// `account_id`, and extract the signer's Ethereum address if the verification passes.
    ///
    /// Typically, the `account_id` would be either the message itself, or be used in one way or
    /// another within the message to validate the signature against.
    ///
    /// This abstraction built with EIP-712 in mind.
    fn verify(account_id: &Self::AccountId, signature: &EcdsaSignature) -> Option<EthereumAddress>;
}

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The signed claim verifier type.
        type Verifier: SignedClaimVerifier<AccountId = Self::AccountId>;

        /// The weight informtation provider type.
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Claim event.
        ClaimAccount {
            /// AccountId that does claiming.
            account_id: T::AccountId,
            /// Ethereum address that is claimed.
            ethereum_address: EthereumAddress,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The native address has already been mapped.
        NativeAddressAlreadyMapped,
        /// The Ethereum address has already been mapped.
        EthereumAddressAlreadyMapped,
        /// Bad ethereum signature.
        BadEthereumSignature,
        /// Invalid ethereum signature.
        InvalidEthereumSignature,
    }

    /// `EthereumAddress` -> `AccountId` storage map.
    #[pallet::storage]
    #[pallet::getter(fn accounts)]
    pub type Accounts<T: Config> =
        StorageMap<_, Twox64Concat, EthereumAddress, T::AccountId, OptionQuery>;

    /// `AccountId` -> `EthereumAddress` storage map.
    #[pallet::storage]
    #[pallet::getter(fn ethereum_addresses)]
    pub type EthereumAddresses<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, EthereumAddress, OptionQuery>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// The mappings to set at genesis.
        pub mappings: Vec<(T::AccountId, EthereumAddress)>,
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            for (account_id, ethereum_address) in &self.mappings {
                Accounts::<T>::insert(ethereum_address, account_id);
                EthereumAddresses::<T>::insert(account_id, ethereum_address);
            }
        }
    }

    #[pallet::call(weight(T::WeightInfo))]
    impl<T: Config> Pallet<T> {
        /// Create a permanent two-way binding between an Ethereum address and a native address.
        /// The native address of the exstrinsic signer is used as a native address, while
        /// the address of the payload signature creator is used as Ethereum address.
        #[pallet::call_index(0)]
        pub fn claim_account(
            origin: OriginFor<T>,
            // Due to the fact that ethereum address can be extracted from any signature
            // we must ensure that the address we've got matches the requested one.
            // The address that was used to be claimed.
            ethereum_address: EthereumAddress,
            ecdsa_signature: EcdsaSignature,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                !EthereumAddresses::<T>::contains_key(&who),
                Error::<T>::NativeAddressAlreadyMapped
            );

            ensure!(
                !Accounts::<T>::contains_key(ethereum_address),
                Error::<T>::EthereumAddressAlreadyMapped
            );

            let expected_ethereum_address = T::Verifier::verify(&who, &ecdsa_signature)
                .ok_or(Error::<T>::BadEthereumSignature)?;

            ensure!(
                ethereum_address == expected_ethereum_address,
                Error::<T>::InvalidEthereumSignature
            );

            Accounts::<T>::insert(ethereum_address, &who);
            EthereumAddresses::<T>::insert(&who, ethereum_address);

            Self::deposit_event(Event::ClaimAccount {
                account_id: who,
                ethereum_address,
            });

            Ok(())
        }
    }
}
