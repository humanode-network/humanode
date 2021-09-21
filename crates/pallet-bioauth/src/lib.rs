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

/// A trait that enables a third-party type to define a potentially fallible conversion from A to B.
/// Is in analogous to [`sp_runtime::Convert`] is a sense that the third-party is acting as
/// the converter, and to [`std::convert::TryFtom`] in a sense that the converion is fallible.
pub trait TryConvert<A, B> {
    /// The error that can occur during conversion.
    type Error;

    /// Take A and return B on success, or an Error if the conversion fails.
    fn try_convert(value: A) -> Result<B, Self::Error>;
}

/// Provides the capability to update the current validators set.
pub trait ValidatorSetUpdater<T> {
    /// Updated the validators set for the of consensus.
    fn update_validators_set<'a, I: Iterator<Item = &'a T> + 'a>(validator_public_keys: I)
    where
        T: 'a;
}

/// Authentication extrinsic playload.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug)]
pub struct Authenticate<OpaqueAuthTicket, Commitment> {
    /// An auth ticket.
    pub ticket: OpaqueAuthTicket,
    /// The robonode signatrure for the opaque auth ticket.
    pub ticket_signature: Commitment,
}

/// The state that we keep in the blockchain for the authorized auth tickets.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Default, Clone, Encode, Decode, Hash, Debug)]
pub struct StoredAuthTicket<PublicKey> {
    /// The public key of a validator that was authorized by a robonode.
    pub public_key: PublicKey,
    /// The nonce that the robonode has provided.
    pub nonce: Vec<u8>,
}

/// The state that we keep in the blockchain for the public keys expiration information.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Default, Clone, Encode, Decode, Hash, Debug)]
pub struct StoredPublicKey<PublicKey, ExpirationTime> {
    /// The public key of a validator.
    pub public_key: PublicKey,
    /// The public key's expiration time.
    pub expiration_time: ExpirationTime,
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
    use crate::{StoredAuthTicket, StoredPublicKey, TryConvert, ValidatorSetUpdater, Verifier};

    use super::Authenticate;
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*, storage::types::ValueQuery};
    use frame_system::pallet_prelude::*;
    use sp_runtime::app_crypto::MaybeHash;
    use sp_std::prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The type of the robonode signature.
        type RobonodeSignature: Member + Parameter;

        /// The public key of the robonode.
        type RobonodePublicKey: Member
            + Parameter
            + MaybeSerializeDeserialize
            + Verifier<Self::RobonodeSignature>
            + Default;

        /// The public key of the validator.
        type ValidatorPublicKey: Member + Parameter + MaybeSerializeDeserialize + MaybeHash;

        /// The opaque auth ticket type.
        type OpaqueAuthTicket: Parameter + AsRef<[u8]> + Send + Sync;

        /// A converter from an opaque to a stored auth ticket.
        type AuthTicketCoverter: TryConvert<
            Self::OpaqueAuthTicket,
            StoredAuthTicket<Self::ValidatorPublicKey>,
        >;

        type LifeTime: Get<Self::BlockNumber>;

        /// The validator set updater to invoke at auth the ticket acceptace.
        type ValidatorSetUpdater: ValidatorSetUpdater<Self::ValidatorPublicKey>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// A list of the authorized auth tickets.
    #[pallet::storage]
    #[pallet::getter(fn stored_nonces)]
    pub type StoredNonces<T> = StorageValue<_, Vec<Vec<u8>>, ValueQuery>;

    /// The public key of the robonode.
    #[pallet::storage]
    #[pallet::getter(fn robonode_public_key)]
    pub type RobonodePublicKey<T> = StorageValue<_, <T as Config>::RobonodePublicKey>;

    /// The public keys expiration information
    #[pallet::storage]
    #[pallet::getter(fn stored_public_keys)]
    pub type StoredPublicKeys<T: Config> = StorageValue<
        _,
        Vec<StoredPublicKey<<T as Config>::ValidatorPublicKey, T::BlockNumber>>,
        ValueQuery,
    >;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub stored_nonces: Vec<Vec<u8>>,
        pub stored_public_keys: Vec<StoredPublicKey<T::ValidatorPublicKey, T::BlockNumber>>,
        pub robonode_public_key: T::RobonodePublicKey,
    }

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                stored_nonces: Default::default(),
                stored_public_keys: Default::default(),
                robonode_public_key: Default::default(),
            }
        }
    }

    // The build of genesis for the pallet.
    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            <StoredNonces<T>>::put(&self.stored_nonces);
            <StoredPublicKeys<T>>::put(&self.stored_public_keys);
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
        AuthTicketStored(StoredAuthTicket<T::ValidatorPublicKey>),
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

    pub enum AuthenticationAttemptValidationError<'a, T: Config> {
        NonceConflict,
        ConflitingPublicKeys(&'a StoredPublicKey<T::ValidatorPublicKey, T::BlockNumber>),
    }

    pub fn validate_authentication_attempt<'a, T: Config>(
        existing_public_keys: &'a [StoredPublicKey<T::ValidatorPublicKey, T::BlockNumber>],
        existing_nonces: &'a [Vec<u8>],
        new: &StoredAuthTicket<T::ValidatorPublicKey>,
    ) -> Result<(), AuthenticationAttemptValidationError<'a, T>> {
        for existing_nonce in existing_nonces.iter() {
            if existing_nonce == &new.nonce {
                return Err(AuthenticationAttemptValidationError::NonceConflict);
            }
        }
        for existing_key in existing_public_keys.iter() {
            if existing_key.public_key == new.public_key {
                return Err(AuthenticationAttemptValidationError::ConflitingPublicKeys(
                    existing_key,
                ));
            }
        }

        Ok(())
    }

    /// Dispatchable functions allows users to interact with the pallet and invoke state changes.
    /// These functions materialize as "extrinsics", which are often compared to transactions.
    /// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn authenticate(
            origin: OriginFor<T>,
            req: Authenticate<T::OpaqueAuthTicket, T::RobonodeSignature>,
        ) -> DispatchResult {
            ensure_none(origin)?;

            let stored_auth_ticket = Self::extract_auth_ticket_checked(req)?;
            let event_stored_auth_ticket = stored_auth_ticket.clone();

            let block_number = frame_system::Pallet::<T>::block_number();
            let public_key = stored_auth_ticket.public_key.clone();

            // Update storage.
            <StoredNonces<T>>::try_mutate::<_, Error<T>, _>(move |nonces_list| {
                <StoredPublicKeys<T>>::try_mutate(move |keys_list| {
                    match validate_authentication_attempt::<T>(
                        keys_list,
                        nonces_list,
                        &stored_auth_ticket,
                    ) {
                        Err(AuthenticationAttemptValidationError::NonceConflict) => {
                            Err(Error::<T>::NonceAlreadyUsed)
                        }
                        Err(AuthenticationAttemptValidationError::ConflitingPublicKeys(_)) => {
                            Err(Error::<T>::PublicKeyAlreadyUsed)
                        }
                        Ok(()) => {
                            nonces_list.push(stored_auth_ticket.nonce);
                            keys_list.push(StoredPublicKey {
                                public_key,
                                expiration_time: T::LifeTime::get() + block_number,
                            });
                            Self::issue_validators_set_update(keys_list.as_slice());
                            Ok(())
                        }
                    }
                })?;
                Ok(())
            })?;

            // Emit an event.
            Self::deposit_event(Event::AuthTicketStored(event_stored_auth_ticket));

            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // Remove all outdated tickets.
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            let current_expiration = <StoredPublicKeys<T>>::get();
            let mut new_expiration = Vec::new();
            for public_key_expiration in current_expiration.iter() {
                if public_key_expiration.expiration_time > n {
                    new_expiration.push(public_key_expiration.clone());
                }
            }
            Self::issue_validators_set_update(new_expiration.as_slice());
            <StoredPublicKeys<T>>::put(new_expiration);
            0
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn extract_auth_ticket_checked(
            req: Authenticate<T::OpaqueAuthTicket, T::RobonodeSignature>,
        ) -> Result<StoredAuthTicket<T::ValidatorPublicKey>, Error<T>> {
            let robonode_public_key =
                RobonodePublicKey::<T>::get().ok_or(Error::<T>::RobonodePublicKeyIsAbsent)?;

            let signature_valid = robonode_public_key
                .verify(&req.ticket, req.ticket_signature)
                .map_err(|_| Error::<T>::UnableToValidateAuthTicketSignature)?;

            if !signature_valid {
                return Err(Error::<T>::AuthTicketSignatureInvalid);
            }

            let auth_ticket = <T::AuthTicketCoverter as TryConvert<_, _>>::try_convert(req.ticket)
                .map_err(|_| Error::<T>::UnableToParseAuthTicket)?;

            Ok(auth_ticket)
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

            let nonces_list = StoredNonces::<T>::get();
            let keys_list = StoredPublicKeys::<T>::get();

            validate_authentication_attempt::<T>(&keys_list, &nonces_list, &stored_auth_ticket)
                .map_err(|_| {
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

        fn issue_validators_set_update(
            public_keys_expiration: &[StoredPublicKey<T::ValidatorPublicKey, T::BlockNumber>],
        ) {
            let validator_public_keys = public_keys_expiration
                .iter()
                .map(|ticket| &ticket.public_key);
            T::ValidatorSetUpdater::update_validators_set(validator_public_keys);
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
