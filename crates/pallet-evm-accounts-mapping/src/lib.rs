//! A substrate pallet that provides logic to map Substrate and EVM accounts.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
pub use pallet::*;

/// An EVM address.
pub type EvmAddress = sp_core::H160;

/// A signature (a 512-bit value, plus 8 bits for recovery ID).
pub type Secp256k1EcdsaSignature = [u8; 65];

/// The verifier for the ethereum signature.
pub trait SignedClaimVerifier {
    /// The type of the native account on the chain.
    type AccountId;

    /// Verify the provided `signature` against a message declaring a claim of the provided
    /// `account_id`, and extract the signer's EVM address if the verification passes.
    ///
    /// Typically, the `account_id` would be either the message itself, or be used in one way or
    /// another within the message to validate the signature against.
    ///
    /// This abstraction built with EIP-712 in mind.
    fn verify(
        account_id: Self::AccountId,
        signature: Secp256k1EcdsaSignature,
    ) -> Option<EvmAddress>;
}

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The signed claim verifier type.
        type Verifier: SignedClaimVerifier<AccountId = Self::AccountId>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Claim event.
        ClaimAccount {
            /// AccountId that does claiming.
            account_id: T::AccountId,
            /// EVM address that is claimed.
            evm_address: EvmAddress,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The native address has already been mapped.
        NativeAddressAlreadyMapped,
        /// The EVM address has already been mapped.
        EvmAddressAlreadyMapped,
        /// Bad ethereum signature.
        BadEthereumSignature,
        /// Invalid ethereum signature.
        InvalidEthereumSignature,
    }

    /// [`EvmAddress`] -> [`AccountId`] storage map.
    #[pallet::storage]
    #[pallet::getter(fn accounts)]
    pub type Accounts<T: Config> =
        StorageMap<_, Twox64Concat, EvmAddress, T::AccountId, OptionQuery>;

    /// [`AccountId`] -> [`EvmAddress`] storage map.
    #[pallet::storage]
    #[pallet::getter(fn evm_addresses)]
    pub type EvmAddresses<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, EvmAddress, OptionQuery>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// The mappings to set at genesis.
        pub mappings: Vec<(T::AccountId, EvmAddress)>,
    }

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                mappings: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            for (account_id, evm_address) in &self.mappings {
                Accounts::<T>::insert(evm_address, account_id);
                EvmAddresses::<T>::insert(account_id, evm_address);
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a permanent two-way binding between an Ethereum address and a native address.
        /// The native address of the exstrinsic signer is used as a native address, while
        /// the address of the payload signature creator is used as Ethereum address.
        #[pallet::weight(10_000)]
        pub fn claim_account(
            origin: OriginFor<T>,
            // According to the fact that evm address can be extracted from any signature,
            // we should clarify that we've got a proper one evm address.
            // The address that was used to be claimed.
            evm_address: EvmAddress,
            signature: Secp256k1EcdsaSignature,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                !EvmAddresses::<T>::contains_key(&who),
                Error::<T>::NativeAddressAlreadyMapped
            );

            ensure!(
                !Accounts::<T>::contains_key(evm_address),
                Error::<T>::EvmAddressAlreadyMapped
            );

            let expected_evm_address = T::Verifier::verify(who.clone(), signature)
                .ok_or(Error::<T>::BadEthereumSignature)?;

            ensure!(
                evm_address == expected_evm_address,
                Error::<T>::InvalidEthereumSignature
            );

            Accounts::<T>::insert(evm_address, &who);
            EvmAddresses::<T>::insert(&who, evm_address);

            Self::deposit_event(Event::ClaimAccount {
                account_id: who,
                evm_address,
            });

            Ok(())
        }
    }
}
