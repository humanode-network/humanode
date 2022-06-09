//! A substrate pallet that provides logic to map Substrate and EVM accounts.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{pallet_prelude::*, sp_runtime::traits::Zero};
use frame_system::pallet_prelude::*;
pub use pallet::*;

pub mod eip_712;

/// Evm address type.
pub type EvmAddress = sp_core::H160;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {

    use eip_712::Verifier;

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_evm::Config {
        /// Event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Verifier type to extract author of EIP-712 claim signature.
        type Eip712Verifier: eip_712::Verifier;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Claim event.
        ClaimAccount {
            /// AccountId that does claiming.
            account_id: T::AccountId,
            /// Eth address that is claimed.
            evm_address: EvmAddress,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// AccountId has already mapped.
        AccountIdAlreadyMapped,
        /// Eth address has already mapped.
        EthAddressAlreadyMapped,
        /// Bad ethereum signature.
        BadSignature,
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

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Extrinsic that provides a possibility to claim eth address once.
        #[pallet::weight(10_000)]
        pub fn claim_account(
            origin: OriginFor<T>,
            signature: eip_712::Signature,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                !EvmAddresses::<T>::contains_key(&who),
                Error::<T>::AccountIdAlreadyMapped
            );

            let account_claim = eip_712::AccountClaimTypedData {
                domain_type: eip_712::DOMAIN_TYPE,
                name: eip_712::NAME,
                version: eip_712::VERSION,
                chain_id: T::ChainId::get(),
                genesis_block_hash: frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero())
                    .as_ref()
                    .to_vec(),
                claim_type: eip_712::CLAIM_TYPE,
                account: who.encode(),
            };

            let eth_address = <T as Config>::Eip712Verifier::verify(account_claim, signature)
                .ok_or(Error::<T>::BadSignature)?;

            ensure!(
                !Accounts::<T>::contains_key(eth_address),
                Error::<T>::EthAddressAlreadyMapped
            );

            Accounts::<T>::insert(eth_address, &who);
            EvmAddresses::<T>::insert(&who, eth_address);

            Self::deposit_event(Event::ClaimAccount {
                account_id: who,
                evm_address: eth_address,
            });

            Ok(())
        }
    }
}
