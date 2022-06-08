//! A substrate pallet that provides logic to map Substrate and EVM accounts.

#![cfg_attr(not(feature = "std"), no_std)]
// Fix clippy for sp_api::decl_runtime_apis!
#![allow(clippy::too_many_arguments, clippy::unnecessary_mut_passed)]

use frame_support::{pallet_prelude::*, sp_runtime::traits::Zero};
use frame_system::pallet_prelude::*;
pub use pallet::*;

pub mod account_claim_eip_712;

/// Evm address type.
pub type EvmAddress = sp_core::H160;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::unused_unit
)]
#[frame_support::pallet]
pub mod pallet {

    use account_claim_eip_712::Verifier;

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_evm::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Eip712Verifier: account_claim_eip_712::Verifier;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        ClaimAccount {
            account_id: T::AccountId,
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
        /// Invalid ethereum sugnature.
        InvalidSignature,
        /// Invalid EIP-712 claim typed data.
        InvalidEip712ClaimData,
    }

    #[pallet::storage]
    #[pallet::getter(fn accounts)]
    pub type Accounts<T: Config> =
        StorageMap<_, Twox64Concat, EvmAddress, T::AccountId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn evm_addresses)]
    pub type EvmAddresses<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, EvmAddress, OptionQuery>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn claim_account(
            origin: OriginFor<T>,
            eth_address: EvmAddress,
            signature: account_claim_eip_712::Signature,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                !EvmAddresses::<T>::contains_key(&who),
                Error::<T>::AccountIdAlreadyMapped
            );
            ensure!(
                !Accounts::<T>::contains_key(eth_address),
                Error::<T>::EthAddressAlreadyMapped
            );

            let account_claim = account_claim_eip_712::AccountClaimTypedData {
                name: account_claim_eip_712::NAME,
                version: account_claim_eip_712::VERSION,
                chain_id: T::ChainId::get(),
                genesis_block_hash: frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero())
                    .as_ref()
                    .to_vec(),
                account: who.encode(),
            };

            let address = <T as Config>::Eip712Verifier::verify(account_claim, signature)
                .map_err(|_| Error::<T>::InvalidEip712ClaimData)?
                .ok_or(Error::<T>::BadSignature)?;

            ensure!(eth_address == address, Error::<T>::InvalidSignature);

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
