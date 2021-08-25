//! A substrate pallet containing the bioauth integration.

#![warn(missing_docs, clippy::clone_on_ref_ptr)]
#![cfg_attr(not(feature = "std"), no_std)]
// Fix clippy for sp_api::decl_runtime_apis!
#![allow(clippy::too_many_arguments, clippy::unnecessary_mut_passed)]

use codec::{Decode, Encode};
use frame_support::traits::IsSubType;
use frame_support::weights::DispatchInfo;
pub use pallet::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
    traits::{DispatchInfoOf, Dispatchable, SignedExtension},
    transaction_validity::{TransactionValidity, TransactionValidityError},
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
/// A non-async (blocking) variant, for use at runtime.
pub trait Verifier<S: ?Sized> {
    /// Verification error.
    /// Error may originate from communicating with HSM, or from a thread pool failure, etc.
    type Error;

    /// Verify that provided data is indeed correctly signed with the provided
    /// signature.
    fn verify<'a, D>(&self, data: D, signature: S) -> Result<bool, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a;
}

sp_api::decl_runtime_apis! {

    /// We need to provide a trait using decl_runtime_apis! macro to be able to call required methods
    /// from external sources using client and runtime_api().
    pub trait BioauthApi {

        /// Get existing stored tickets for current block.
        fn stored_auth_tickets() -> Vec<StoredAuthTicket>;
    }
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
    use sp_runtime::traits::Convert;
    use sp_std::prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_aura::Config
        + for<'a> Convert<&'a [u8], <Self as pallet_aura::Config>::AuthorityId>
    {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The public key of the robonode.
        type RobonodePublicKey: Verifier<Vec<u8>> + codec::FullCodec + Default + serde_nostd::SerDe;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// A list of the authorized auth tickets.
    #[pallet::storage]
    #[pallet::getter(fn stored_auth_tickets)]
    pub type StoredAuthTickets<T> = StorageValue<_, Vec<StoredAuthTicket>, ValueQuery>;

    /// The public key of the robonode.
    #[pallet::storage]
    #[pallet::getter(fn robonode_public_key)]
    pub type RobonodePublicKey<T> = StorageValue<_, <T as Config>::RobonodePublicKey>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub stored_auth_tickets: Vec<StoredAuthTicket>,
        pub robonode_public_key: <T as Config>::RobonodePublicKey,
    }

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                stored_auth_tickets: Default::default(),
                robonode_public_key: Default::default(),
            }
        }
    }

    // The build of genesis for the pallet.
    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            <StoredAuthTickets<T>>::put(&self.stored_auth_tickets);
            <RobonodePublicKey<T>>::put(&self.robonode_public_key);
        }
    }

    // Pallets use events to inform users when important changes are made.
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [stored_auth_ticket]
        AuthTicketStored(StoredAuthTicket),
    }

    /// Possible error conditions during `authenticate` call processing.
    #[pallet::error]
    pub enum Error<T> {
        /// The robonode public key is not at the chain state.
        RobonodePublicKeyIsAbsent,
        /// We were unable to validate the signature, i.e. it is unclear whether it is valid or
        /// not.
        UnableToValidateAuthTicketSignature,
        /// The signature for the auth ticket is invalid.
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
            ensure_none(origin)?;

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
                        Self::update_aura(list.as_slice());
                        Ok(())
                    }
                }
            })?;

            // Emit an event.
            Self::deposit_event(Event::AuthTicketStored(event_stored_auth_ticket));

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn extract_auth_ticket_checked(
            req: Authenticate,
        ) -> Result<StoredAuthTicket, Error<T>> {
            let robonode_public_key =
                RobonodePublicKey::<T>::get().ok_or(Error::<T>::RobonodePublicKeyIsAbsent)?;
            let signature_valid = robonode_public_key
                .verify(&req.ticket, req.ticket_signature.clone())
                .map_err(|_| Error::<T>::UnableToValidateAuthTicketSignature)?;
            if !signature_valid {
                return Err(Error::<T>::AuthTicketSignatureInvalid);
            }

            let opaque_auth_ticket = OpaqueAuthTicket::from(req.ticket);

            let auth_ticket: AuthTicket = (&opaque_auth_ticket)
                .try_into()
                .map_err(|_| Error::<T>::UnableToParseAuthTicket)?;

            Ok(auth_ticket.into())
        }

        pub fn check_tx(call: &Call<T>) -> TransactionValidity {
            let transaction = match call {
                Call::authenticate(ref transaction) => transaction,
                // Deny all unknown transactions.
                _ => {
                    // The only supported transaction by this pallet is `authenticate`, so anything
                    // else is illegal.
                    return Err(TransactionValidityError::Invalid(InvalidTransaction::Call));
                }
            };

            let stored_auth_ticket = Self::extract_auth_ticket_checked(transaction.clone())
                .map_err(|error| {
                    frame_support::sp_tracing::error!(
                        message = "Auth Ticket could not be extracted",
                        ?error
                    );
                    // Use custom code 's' for "signature" error.
                    TransactionValidityError::Invalid(InvalidTransaction::Custom(b's'))
                })?;

            let list = StoredAuthTickets::<T>::get();

            validate_authentication_attempt(&list, &stored_auth_ticket).map_err(|_e| {
                // Use custom code 'c' for "conflict" error.
                TransactionValidityError::Invalid(InvalidTransaction::Custom(b'c'))
            })?;

            // We must use non-default [`TransactionValidity`] here.
            ValidTransaction::with_tag_prefix("bioauth")
                // Apparently tags are required for the tx pool to build a chain of transactions;
                // in our case, we the structure of the [`StoredAuthTickets`] is supposed to be
                // unordered, and act like a CRDT.
                // TODO: ensure we have the unordered (CRDT) semantics for the [`authenticate`] txs.
                .and_provides(stored_auth_ticket)
                .priority(50)
                .longevity(1)
                .propagate(true)
                .build()
        }

        fn update_aura(list: &[StoredAuthTicket])
        where
            T: pallet_aura::Config,
        {
            let authorities = list
                .iter()
                .map(|ticket| T::convert(ticket.public_key.as_slice()))
                .collect::<Vec<_>>();
            pallet_aura::Authorities::<T>::set(authorities);
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(
            _source: TransactionSource,
            _call: &Self::Call,
        ) -> TransactionValidity {
            // Allow all transactions from this pallet, and delegate the actual logic to the
            // SignedExtension implementation logic.
            // See https://github.com/paritytech/substrate/issues/3419
            Ok(Default::default())
        }
    }
}

/// Checks the validity of the unsigned [`Call::authenticate`] tx.
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
        match call.is_sub_type() {
            Some(call) => Pallet::<T>::check_tx(call),
            _ => Ok(Default::default()),
        }
    }

    fn validate_unsigned(
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        match call.is_sub_type() {
            Some(call) => Pallet::<T>::check_tx(call),
            _ => Ok(Default::default()),
        }
    }
}
