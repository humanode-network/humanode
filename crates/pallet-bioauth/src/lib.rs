//! A substrate pallet containing the bioauth integration.

#![warn(missing_docs, clippy::clone_on_ref_ptr)]
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::traits::IsSubType;
use frame_support::weights::DispatchInfo;
pub use pallet::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
    traits::{DispatchInfoOf, Dispatchable, SignedExtension},
    transaction_validity::{InvalidTransaction, TransactionValidity, TransactionValidityError},
};
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// Authentication extrinsic playload.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug)]
pub struct Authenticate {
    /// The opaque auth ticket.
    pub ticket: Vec<u8>,
    /// The robonode signatrure for the opaque auth ticket.
    pub ticket_signature: Vec<u8>,
}

/// The state that we keep in the blockchain for the authorized authentication tickets.
///
/// It is decoupled from the [`primitives_auth_ticket::AuthTicket`], such that it's possible to version
/// and update those independently.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug)]
pub struct StoredAuthTicket {
    /// The public key of the validator.
    pub public_key: Vec<u8>,
    /// A one-time use value.
    pub nonce: Vec<u8>,
}

impl From<primitives_auth_ticket::AuthTicket> for StoredAuthTicket {
    fn from(val: primitives_auth_ticket::AuthTicket) -> Self {
        Self {
            public_key: val.public_key,
            nonce: val.authentication_nonce,
        }
    }
}

/// Verifier provides the verification of the data accompanied with the
/// signature or proof data.
pub trait Verifier {
    /// Verify that provided data is indeed correctly signed with the provided
    /// signature.
    fn verify<D: AsRef<[u8]>, S: AsRef<[u8]>>(&self, data: &D, signature: &S) -> bool;
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
    use core::convert::TryInto;

    use super::{Authenticate, StoredAuthTicket, Verifier};
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*, storage::types::ValueQuery};
    use frame_system::pallet_prelude::*;
    use primitives_auth_ticket::{AuthTicket, OpaqueAuthTicket};
    use sp_std::prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type RobonodeSignatureVerifier: Verifier + Encode;

        #[pallet::constant]
        type RobonodeSignatureVerifierInstance: Get<Self::RobonodeSignatureVerifier>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// A list of the authorized auth tickets.
    #[pallet::storage]
    #[pallet::getter(fn stored_auth_tickets)]
    pub type StoredAuthTickets<T> = StorageValue<_, Vec<StoredAuthTicket>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub stored_auth_tickets: Vec<StoredAuthTicket>,
    }

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                stored_auth_tickets: Default::default(),
            }
        }
    }

    // The build of genesis for the pallet.
    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            <StoredAuthTickets<T>>::put(&self.stored_auth_tickets);
        }
    }

    // Pallets use events to inform users when important changes are made.
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        AuthTicketStored(StoredAuthTicket, T::AccountId),
    }

    /// Possible error conditions during `authenticate` call processing.
    #[pallet::error]
    pub enum Error<T> {
        /// The signature for the auth ticket did not validate.
        AuthTicketSignatureInvalid,
        /// Unable to parse the auth ticket.
        UnableToParseAuthTicket,
        /// This nonce has already been seen by the network.
        NonceAlreadyUsed,
        /// This public key has already been used.
        PublicKeyAlreadyUsed,
    }

    pub enum AuthenticationAttemptValidationError<'a> {
        NonceConflict,
        ConflitingPublicKeys(Vec<&'a StoredAuthTicket>),
    }

    pub fn validate_authentication_attempt<'a>(
        existing: &'a [StoredAuthTicket],
        new: &StoredAuthTicket,
    ) -> Result<(), AuthenticationAttemptValidationError<'a>> {
        let mut conflicting_tickets = Vec::new();
        for existing in existing.iter() {
            if existing.nonce == new.nonce {
                return Err(AuthenticationAttemptValidationError::NonceConflict);
            }
            if existing.public_key == new.public_key {
                conflicting_tickets.push(existing);
            }
        }

        if !conflicting_tickets.is_empty() {
            return Err(AuthenticationAttemptValidationError::ConflitingPublicKeys(
                conflicting_tickets,
            ));
        }

        Ok(())
    }

    /// Dispatchable functions allows users to interact with the pallet and invoke state changes.
    /// These functions materialize as "extrinsics", which are often compared to transactions.
    /// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn authenticate(origin: OriginFor<T>, req: Authenticate) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let stored_auth_ticket = Self::extract_auth_ticket_checked(req)?;
            let event_stored_auth_ticket = stored_auth_ticket.clone();

            // Update storage.
            <StoredAuthTickets<T>>::try_mutate(move |list| {
                match validate_authentication_attempt(list, &stored_auth_ticket) {
                    Err(AuthenticationAttemptValidationError::NonceConflict) => {
                        Err(Error::<T>::NonceAlreadyUsed)
                    }
                    Err(AuthenticationAttemptValidationError::ConflitingPublicKeys(_)) => {
                        Err(Error::<T>::PublicKeyAlreadyUsed)
                    }
                    Ok(()) => {
                        // Authentication was successfull, add the incoming auth ticket to the list.
                        list.push(stored_auth_ticket);
                        Ok(())
                    }
                }
            })?;

            // Emit an event.
            Self::deposit_event(Event::AuthTicketStored(event_stored_auth_ticket, who));

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn extract_auth_ticket_checked(
            req: Authenticate,
        ) -> Result<StoredAuthTicket, Error<T>> {
            let robonode_public_key = T::RobonodeSignatureVerifierInstance::get();
            if !robonode_public_key.verify(&req.ticket, &req.ticket_signature) {
                return Err(Error::<T>::AuthTicketSignatureInvalid);
            }

            let opaque_auth_ticket = OpaqueAuthTicket::from(req.ticket);

            let auth_ticket: AuthTicket = (&opaque_auth_ticket)
                .try_into()
                .map_err(|_| Error::<T>::UnableToParseAuthTicket)?;

            Ok(auth_ticket.into())
        }
    }
}

// The following section implements the `SignedExtension` trait
// for the `CheckBioauthTx` type.

/// The `CheckBioauthTx` struct.
#[derive(Encode, Decode, Clone, Eq, PartialEq, Default)]
pub struct CheckBioauthTx<T: Config + Send + Sync>(PhantomData<T>);

/// Debug impl for the `CheckBioauthTx` struct.
impl<T: Config + Send + Sync> Debug for CheckBioauthTx<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "CheckBioauthTx")
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

/// Implementation of the `SignedExtension` trait for the `CheckBioauthTx` struct.
impl<T: Config + Send + Sync> SignedExtension for CheckBioauthTx<T>
where
    T::Call: Dispatchable<Info = DispatchInfo>,
    <T as frame_system::Config>::Call: IsSubType<Call<T>>,
{
    type AccountId = T::AccountId;
    type Call = T::Call;
    type AdditionalSigned = ();
    type Pre = ();
    const IDENTIFIER: &'static str = "CheckBioauthTx";

    fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
        Ok(())
    }

    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        let _account_id = who;

        // check for `authenticate`
        match call.is_sub_type() {
            Some(Call::authenticate(ref transaction)) => {
                // We need to call our validate_bioauth from pallet
                let stored_auth_ticket = Pallet::<T>::extract_auth_ticket_checked(
                    transaction.clone(),
                )
                .map_err(|_e| TransactionValidityError::Invalid(InvalidTransaction::Call))?;

                let list = StoredAuthTickets::<T>::get();

                validate_authentication_attempt(&list, &stored_auth_ticket)
                    .map_err(|_e| TransactionValidityError::Invalid(InvalidTransaction::Call))?;

                Ok(Default::default())
            }
            _ => Ok(Default::default()),
        }
    }
}
