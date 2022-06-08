//! A substrate pallet that provides logic to map Substrate and EVM accounts.

#![cfg_attr(not(feature = "std"), no_std)]
// Fix clippy for sp_api::decl_runtime_apis!
#![allow(clippy::too_many_arguments, clippy::unnecessary_mut_passed)]

use frame_support::{pallet_prelude::*, sp_runtime::traits::Zero};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use sp_core::H256;
use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};
use sp_std::prelude::*;

/// Evm address type.
pub type EvmAddress = sp_core::H160;

/// A signature (a 512-bit value, plus 8 bits for recovery ID).
pub type Eip712Signature = [u8; 65];

/// Provides the capability to verify an Eip712 based ethereum signature.
pub trait Eip712Verifier {
    /// Verify the signature and extract a corresponding [`EvmAddress`] if it's ok.
    fn verify(
        domain_separator: [u8; 32],
        message: Vec<u8>,
        signature: Eip712Signature,
    ) -> Option<EvmAddress>;
}

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::unused_unit
)]
#[frame_support::pallet]
pub mod pallet {

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Chain ID of EVM.
        #[pallet::constant]
        type ChainId: Get<u64>;

        type Eip712Verifier: Eip712Verifier;
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
            signature: Eip712Signature,
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
                Self::account_domain_separator(),
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
    /// A domain separator used at Eip712 flow.
    fn account_domain_separator() -> [u8; 32] {
        let domain_hash =
            keccak_256(b"EIP712Domain(string name,string version,uint256 chainId,bytes32 salt)");
        let mut domain_seperator_msg = domain_hash.to_vec();
        domain_seperator_msg.extend_from_slice(&keccak_256(b"Humanode EVM claim")); // name
        domain_seperator_msg.extend_from_slice(&keccak_256(b"1")); // version
        domain_seperator_msg.extend_from_slice(&keccak_256(b"5234")); // chain id
        domain_seperator_msg.extend_from_slice(
            frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero()).as_ref(),
        ); // genesis block hash
        keccak_256(domain_seperator_msg.as_slice())
    }
}

/// Verify Eip712 typed signature based on provided domain_separator and entire message.
pub struct Eip712VerifierFactory;

impl Eip712Verifier for Eip712VerifierFactory {
    fn verify(
        domain_separator: [u8; 32],
        message: Vec<u8>,
        signature: Eip712Signature,
    ) -> Option<EvmAddress> {
        let msg = Self::eip712_signable_message(domain_separator, message);
        let msg_hash = keccak_256(msg.as_slice());

        recover_signer(&signature, &msg_hash)
    }
}

impl Eip712VerifierFactory {
    /// Eip-712 message to be signed.
    fn eip712_signable_message(domain_separator: [u8; 32], message: Vec<u8>) -> Vec<u8> {
        let payload_hash = Self::payload_hash(message);

        let mut msg = b"\x19\x01".to_vec();
        msg.extend_from_slice(&domain_separator);
        msg.extend_from_slice(&payload_hash);
        msg
    }

    /// Get payload hash from message.
    fn payload_hash(message: Vec<u8>) -> [u8; 32] {
        keccak_256(message.as_slice())
    }
}

/// A helper function to return a corresponding [`EvmAddress`] from signature and message hash.
fn recover_signer(sig: &Eip712Signature, msg_hash: &[u8; 32]) -> Option<EvmAddress> {
    secp256k1_ecdsa_recover(sig, msg_hash)
        .map(|pubkey| EvmAddress::from(H256::from_slice(&keccak_256(&pubkey))))
        .ok()
}
