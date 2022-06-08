//! A substrate pallet that provides logic to map Substrate and EVM accounts.

#![cfg_attr(not(feature = "std"), no_std)]
// Fix clippy for sp_api::decl_runtime_apis!
#![allow(clippy::too_many_arguments, clippy::unnecessary_mut_passed)]

use frame_support::{pallet_prelude::*, sp_runtime::traits::Zero};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use sp_io::hashing::keccak_256;

pub mod eip_712;

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

    use eip_712::Verifier;

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_evm::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Eip712Verifier: eip_712::Verifier;
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
            signature: eip_712::Signature,
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

            let address = <T as Config>::Eip712Verifier::verify(
                Self::domain_separator(),
                who.encode(),
                signature,
            )
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

impl<T: Config> Pallet<T> {
    /// A corresponding domain separator used at Eip712 flow.
    fn domain_separator() -> [u8; 32] {
        let domain = eip_712::DOMAIN.as_bytes();
        let name = eip_712::NAME.as_bytes();
        let version = eip_712::VERSION.to_ne_bytes();
        let chain_id = T::ChainId::get().to_ne_bytes();
        let genesis_block_hash = frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero());

        let domain_hash = keccak_256(domain);
        let mut domain_seperator_msg = domain_hash.to_vec();
        domain_seperator_msg.extend_from_slice(&keccak_256(name));
        domain_seperator_msg.extend_from_slice(&keccak_256(&version));
        domain_seperator_msg.extend_from_slice(&keccak_256(&chain_id));
        domain_seperator_msg.extend_from_slice(genesis_block_hash.as_ref());
        keccak_256(domain_seperator_msg.as_slice())
    }
}
