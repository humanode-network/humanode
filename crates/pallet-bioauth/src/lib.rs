//! A substrate pallet containing the bioauth integration.

#![cfg_attr(not(feature = "std"), no_std)]
// Fix clippy for sp_api::decl_runtime_apis!
#![allow(clippy::too_many_arguments, clippy::unnecessary_mut_passed)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::{IsSubType, StorageVersion};
use frame_support::weights::DispatchInfo;
use frame_support::{traits::ConstU32, WeakBoundedVec};
pub use pallet::*;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
    traits::{DispatchInfoOf, Dispatchable, SignedExtension},
    transaction_validity::{TransactionValidity, TransactionValidityError},
};
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;
use sp_std::prelude::*;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

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
    /// Provide an up-to-date the validators set for the of consensus.
    fn update_validators_set<'a, I: Iterator<Item = &'a T> + 'a>(validator_public_keys: I)
    where
        T: 'a;

    /// Provide an initial validators set for the of consensus at genesis.
    fn init_validators_set<'a, I: Iterator<Item = &'a T> + 'a>(validator_public_keys: I)
    where
        T: 'a;
}

impl<T> ValidatorSetUpdater<T> for () {
    fn update_validators_set<'a, I: Iterator<Item = &'a T> + 'a>(_validator_public_keys: I)
    where
        T: 'a,
    {
    }

    fn init_validators_set<'a, I: Iterator<Item = &'a T> + 'a>(_validator_public_keys: I)
    where
        T: 'a,
    {
    }
}

/// Provides the capability to get current moment.
pub trait CurrentMoment<Moment> {
    /// Return current moment.
    fn now() -> Moment;
}

/// Authentication extrinsic playload.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct Authenticate<OpaqueAuthTicket, Commitment> {
    /// An auth ticket.
    pub ticket: OpaqueAuthTicket,
    /// The robonode signatrure for the opaque auth ticket.
    pub ticket_signature: Commitment,
}

/// The maximum length of a single nonce (in bytes).
pub const AUTH_TICKET_NONCE_MAX_BYTES: u32 = 256;

/// The nonce used by robonode in the auth tickets.
pub type AuthTicketNonce = Vec<u8>;

/// The nonce type in this pallet with bounded number of bytes at the nonce.
pub type BoundedAuthTicketNonce = WeakBoundedVec<u8, ConstU32<AUTH_TICKET_NONCE_MAX_BYTES>>;

/// The auth ticket passed to us from the robonode.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct AuthTicket<PublicKey> {
    /// The public key of a validator that was authorized by a robonode.
    pub public_key: PublicKey,
    /// The nonce that the robonode has provided.
    pub nonce: AuthTicketNonce,
}

/// The state that we keep in the blockchain for an active authentication.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo, MaxEncodedLen)]
pub struct Authentication<PublicKey, Moment> {
    /// The public key of a validator.
    pub public_key: PublicKey,
    /// The moment at which the authentication becomes expired.
    pub expires_at: Moment,
}

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

/// Custom invalid transaction error codes.
#[repr(u8)]
pub enum CustomInvalidTransactionCodes {
    /// We were unable to parse the auth ticket.
    /// This happens after the signature has already been verified.
    UnableToParseAuthTicket = b't',
}

/// A hook that runs before the bioauth.
/// You can abort the bioauth here (if needed) by returning an error from the hook.
///
/// This hook runs when we have already verified the auth ticket.
pub trait BeforeAuthHook<PublicKey, Moment> {
    /// The data that this hook want to keep around.
    /// The [`AfterAuthHook`] can later use them.
    type Data;

    /// The hook to run.
    fn hook(
        authentication: &Authentication<PublicKey, Moment>,
    ) -> Result<Self::Data, sp_runtime::DispatchError>;
}

impl<PublicKey, Moment> BeforeAuthHook<PublicKey, Moment> for () {
    type Data = ();

    fn hook(
        _authentication: &Authentication<PublicKey, Moment>,
    ) -> Result<Self::Data, sp_runtime::DispatchError> {
        Ok(())
    }
}

/// A hook that runs after the bioauth.
///
/// Can't abort the bioauth, as it executes after bioauth has already happened.
pub trait AfterAuthHook<BeforeHookData> {
    /// The hook to run.
    /// Takes the data returned by the [`BeforeAuthHook`] as an argument.
    fn hook(before_hook_data: BeforeHookData);
}

impl<BeforeHookData> AfterAuthHook<BeforeHookData> for () {
    fn hook(_before_hook_data: BeforeHookData) {}
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
    use codec::MaxEncodedLen;
    use frame_support::{
        pallet_prelude::*, sp_tracing::error, storage::types::ValueQuery, WeakBoundedVec,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::{app_crypto::MaybeHash, traits::AtLeast32Bit, DispatchError};

    use super::*;
    use crate::weights::WeightInfo;

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
            + Default
            + MaxEncodedLen;

        /// The public key of the validator.
        type ValidatorPublicKey: Member
            + Parameter
            + MaybeSerializeDeserialize
            + MaybeHash
            + MaxEncodedLen;

        /// The opaque auth ticket type.
        type OpaqueAuthTicket: Parameter + AsRef<[u8]> + Send + Sync;

        /// A converter from an opaque to a transparent auth ticket.
        type AuthTicketCoverter: TryConvert<
            Self::OpaqueAuthTicket,
            AuthTicket<Self::ValidatorPublicKey>,
        >;

        /// Type used for expressing timestamp.
        type Moment: Parameter
            + Default
            + AtLeast32Bit
            + Copy
            + MaybeSerializeDeserialize
            + MaxEncodedLen;

        /// Type used for pretty printing the timestamp.
        type DisplayMoment: From<Self::Moment> + core::fmt::Display;

        /// The getter for the current moment.
        type CurrentMoment: CurrentMoment<Self::Moment>;

        /// The amount of time (in moments) after which the authentications expire.
        type AuthenticationsExpireAfter: Get<Self::Moment>;

        /// The validator set updater to invoke at auth the ticket acceptace.
        type ValidatorSetUpdater: ValidatorSetUpdater<Self::ValidatorPublicKey>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// The maximum number of authentications.
        type MaxAuthentications: Get<u32>;

        /// The maximum number of nonces.
        type MaxNonces: Get<u32>;

        /// Before authentication hook.
        type BeforeAuthHook: BeforeAuthHook<Self::ValidatorPublicKey, Self::Moment>;

        /// After authentication hook.
        type AfterAuthHook: AfterAuthHook<
            <Self::BeforeAuthHook as BeforeAuthHook<Self::ValidatorPublicKey, Self::Moment>>::Data,
        >;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// The public key of the robonode.
    #[pallet::storage]
    #[pallet::getter(fn robonode_public_key)]
    pub type RobonodePublicKey<T> = StorageValue<_, <T as Config>::RobonodePublicKey, ValueQuery>;

    /// A list of all consumed nonces.
    #[pallet::storage]
    #[pallet::getter(fn consumed_auth_ticket_nonces)]
    pub type ConsumedAuthTicketNonces<T: Config> =
        StorageValue<_, WeakBoundedVec<BoundedAuthTicketNonce, T::MaxNonces>, ValueQuery>;

    /// A list of all active authentications.
    #[pallet::storage]
    #[pallet::getter(fn active_authentications)]
    pub type ActiveAuthentications<T: Config> = StorageValue<
        _,
        WeakBoundedVec<Authentication<T::ValidatorPublicKey, T::Moment>, T::MaxAuthentications>,
        ValueQuery,
    >;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub robonode_public_key: T::RobonodePublicKey,
        pub consumed_auth_ticket_nonces: Vec<AuthTicketNonce>,
        pub active_authentications: Vec<Authentication<T::ValidatorPublicKey, T::Moment>>,
    }

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                robonode_public_key: Default::default(),
                consumed_auth_ticket_nonces: Default::default(),
                active_authentications: Default::default(),
            }
        }
    }

    // The build of genesis for the pallet.
    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            let bounded_consumed_auth_ticket_nonces = WeakBoundedVec::<_, T::MaxNonces>::try_from(
                self.consumed_auth_ticket_nonces
                    .iter()
                    .cloned()
                    .map(|nonce| {
                        BoundedAuthTicketNonce::try_from(nonce).expect(
                            "Initial nonce len must be less than AUTH_TICKET_NONCE_MAX_BYTES",
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .expect("Initial nonces must be less than T::MaxNonces");

            let bounded_active_authentications =
                WeakBoundedVec::<_, T::MaxAuthentications>::try_from(
                    self.active_authentications.clone(),
                )
                .expect("Initial authentications must be less than T::MaxAuthentications");

            <RobonodePublicKey<T>>::put(&self.robonode_public_key);
            <ConsumedAuthTicketNonces<T>>::put(bounded_consumed_auth_ticket_nonces);
            <ActiveAuthentications<T>>::put(bounded_active_authentications);

            <Pallet<T>>::issue_validators_set_init(&self.active_authentications);
        }
    }

    // Pallets use events to inform users when important changes are made.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// New authentication was added to the state.
        NewAuthentication {
            validator_public_key: T::ValidatorPublicKey,
        },
    }

    /// Possible error conditions during `authenticate` call processing.
    #[pallet::error]
    pub enum Error<T> {
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

    #[derive(Debug)]
    enum AuthenticationAttemptValidationError {
        NonceConflict,
        AlreadyAuthenticated,
    }

    /// Validate the incloming authentication attempt, checking the auth ticket data against
    /// the passed input.
    fn validate_authentication_attempt<'a, T: Config>(
        consumed_auth_ticket_nonces: &'a [BoundedAuthTicketNonce],
        active_authentications: &'a [Authentication<T::ValidatorPublicKey, T::Moment>],
        auth_ticket: &AuthTicket<T::ValidatorPublicKey>,
    ) -> Result<(), AuthenticationAttemptValidationError> {
        for consumed_auth_ticket_nonce in consumed_auth_ticket_nonces.iter() {
            if consumed_auth_ticket_nonce == &auth_ticket.nonce {
                return Err(AuthenticationAttemptValidationError::NonceConflict);
            }
        }
        for active_authentication in active_authentications.iter() {
            if active_authentication.public_key == auth_ticket.public_key {
                return Err(AuthenticationAttemptValidationError::AlreadyAuthenticated);
            }
        }

        Ok(())
    }

    /// Public API the pallet exposes to the runtime.
    impl<T: Config> Pallet<T> {
        pub fn is_authenticated(public_key: &<T as Config>::ValidatorPublicKey) -> bool {
            ActiveAuthentications::<T>::get()
                .iter()
                .any(|authentication| &authentication.public_key == public_key)
        }

        pub fn deauthenticate(public_key: &<T as Config>::ValidatorPublicKey) -> bool {
            ActiveAuthentications::<T>::mutate(|active_authentications| {
                let mut removed = false;
                active_authentications.retain(|authentication| {
                    if &authentication.public_key == public_key {
                        removed = true;
                        return false;
                    }
                    true
                });
                removed
            })
        }
    }

    /// Dispatchable functions allow users to interact with the pallet and invoke state changes.
    /// These functions materialize as "extrinsics", which are often compared to transactions.
    /// Dispatchable functions must be annotated with a weight and must return
    /// a [`frame_support::dispatch::DispatchResult`] or
    /// or [`frame_support::dispatch::DispatchResultWithPostInfo`].
    ///
    /// Weight: `O(M + N) where M is the number of authentications and N is the number of nonces`
    /// Cost incurred from decoding vec of length M or N. Charged as maximum
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(T::WeightInfo::authenticate())]
        pub fn authenticate(
            origin: OriginFor<T>,
            req: Authenticate<T::OpaqueAuthTicket, T::RobonodeSignature>,
        ) -> DispatchResult {
            ensure_none(origin)?;

            let auth_ticket =
                Self::extract_auth_ticket_checked(req).map_err(|error| match error {
                    AuthTicketExtractionError::UnableToValidateSignature => {
                        Error::<T>::UnableToValidateAuthTicketSignature
                    }
                    AuthTicketExtractionError::SignatureInvalid => {
                        Error::<T>::AuthTicketSignatureInvalid
                    }
                    AuthTicketExtractionError::UnableToParse => Error::<T>::UnableToParseAuthTicket,
                })?;
            let public_key = auth_ticket.public_key.clone();

            // Update storage.
            <ConsumedAuthTicketNonces<T>>::try_mutate::<_, DispatchError, _>(
                move |consumed_auth_ticket_nonces| {
                    <ActiveAuthentications<T>>::try_mutate::<_, DispatchError, _>(
                        move |active_authentications| {
                            validate_authentication_attempt::<T>(
                                consumed_auth_ticket_nonces,
                                active_authentications,
                                &auth_ticket,
                            )
                            .map_err(|err| match err {
                                AuthenticationAttemptValidationError::NonceConflict => {
                                    Error::<T>::NonceAlreadyUsed
                                }
                                AuthenticationAttemptValidationError::AlreadyAuthenticated => {
                                    Error::<T>::PublicKeyAlreadyUsed
                                }
                            })?;

                            // Update internal state.
                            let current_moment = T::CurrentMoment::now();
                            let mut updated_consumed_auth_ticket_nonces =
                                consumed_auth_ticket_nonces.clone().into_inner();

                            updated_consumed_auth_ticket_nonces.push(
                                BoundedAuthTicketNonce::force_from(
                                    auth_ticket.nonce,
                                    Some("bioauth::authenticate::auth_ticket_nonce"),
                                ),
                            );

                            *consumed_auth_ticket_nonces =
                                WeakBoundedVec::<_, T::MaxNonces>::force_from(
                                    updated_consumed_auth_ticket_nonces,
                                    Some("bioauth::authenticate::nonces"),
                                );

                            let mut updated_active_authentications =
                                active_authentications.clone().into_inner();
                            let authentication = Authentication {
                                public_key: public_key.clone(),
                                expires_at: current_moment + T::AuthenticationsExpireAfter::get(),
                            };

                            // Run the before hook, abort if needed.
                            let before_hook_data =
                                <T as Config>::BeforeAuthHook::hook(&authentication)?;

                            updated_active_authentications.push(authentication);

                            *active_authentications =
                                WeakBoundedVec::<_, T::MaxAuthentications>::force_from(
                                    updated_active_authentications,
                                    Some("bioauth::authentication::authentications"),
                                );

                            // Issue an update to the external validators set.
                            Self::issue_validators_set_update(active_authentications.as_slice());

                            // Run the after hook.
                            <T as Config>::AfterAuthHook::hook(before_hook_data);

                            // Emit an event.
                            Self::deposit_event(Event::NewAuthentication {
                                validator_public_key: public_key,
                            });
                            Ok(())
                        },
                    )?;
                    Ok(())
                },
            )?;
            Ok(())
        }

        #[pallet::weight(T::WeightInfo::set_robonode_public_key())]
        pub fn set_robonode_public_key(
            origin: OriginFor<T>,
            robonode_public_key: T::RobonodePublicKey,
        ) -> DispatchResult {
            ensure_root(origin)?;
            <RobonodePublicKey<T>>::put(&robonode_public_key);
            <ActiveAuthentications<T>>::put(WeakBoundedVec::default());
            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            // Remove expired authentications.
            let current_moment = T::CurrentMoment::now();
            let possibly_expired_authentications = <ActiveAuthentications<T>>::get();
            let possibly_expired_authentications_len = possibly_expired_authentications.len();
            let active_authentications = possibly_expired_authentications
                .into_iter()
                .filter(|possibly_expired_authentication| {
                    possibly_expired_authentication.expires_at > current_moment
                })
                .collect::<Vec<_>>();

            let update_required =
                possibly_expired_authentications_len != active_authentications.len();
            if update_required {
                // We use force_from and None as a resulted active authentications Vec
                // can't become bigger than it was. Just filtering was done before.
                let bounded_active_authentications =
                    WeakBoundedVec::<_, T::MaxAuthentications>::force_from(
                        active_authentications.clone(),
                        None,
                    );
                Self::issue_validators_set_update(active_authentications.as_slice());
                <ActiveAuthentications<T>>::put(bounded_active_authentications);
            }

            // Weight: O(M) where M is the number of auths.
            // Cost incurred from decoding vec of length M. Charged as maximum.
            T::WeightInfo::on_initialize(update_required)
        }
    }

    #[derive(Debug)]
    enum AuthTicketExtractionError {
        UnableToValidateSignature,
        SignatureInvalid,
        UnableToParse,
    }

    impl<T: Config> Pallet<T> {
        fn extract_auth_ticket_checked(
            req: Authenticate<T::OpaqueAuthTicket, T::RobonodeSignature>,
        ) -> Result<AuthTicket<T::ValidatorPublicKey>, AuthTicketExtractionError> {
            let robonode_public_key = RobonodePublicKey::<T>::get();

            let signature_valid = robonode_public_key
                .verify(&req.ticket, req.ticket_signature)
                .map_err(|_| AuthTicketExtractionError::UnableToValidateSignature)?;

            if !signature_valid {
                return Err(AuthTicketExtractionError::SignatureInvalid);
            }

            let auth_ticket = <T::AuthTicketCoverter as TryConvert<_, _>>::try_convert(req.ticket)
                .map_err(|_| AuthTicketExtractionError::UnableToParse)?;

            Ok(auth_ticket)
        }

        pub fn check_tx(call: &Call<T>) -> TransactionValidity {
            let transaction = match call {
                Call::authenticate { req: transaction } => transaction,
                // Deny all unknown transactions.
                _ => {
                    // The only supported transaction by this pallet is `authenticate`, so anything
                    // else is illegal.
                    return Err(TransactionValidityError::Invalid(InvalidTransaction::Call));
                }
            };

            let auth_ticket =
                Self::extract_auth_ticket_checked(transaction.clone()).map_err(|error| {
                    error!(message = "Auth Ticket could not be extracted", ?error);
                    // Use bad proof error code, as the extraction.
                    TransactionValidityError::Invalid(match error {
                        AuthTicketExtractionError::UnableToValidateSignature
                        | AuthTicketExtractionError::SignatureInvalid => {
                            InvalidTransaction::BadProof
                        }
                        AuthTicketExtractionError::UnableToParse => InvalidTransaction::Custom(
                            CustomInvalidTransactionCodes::UnableToParseAuthTicket as u8,
                        ),
                    })
                })?;

            let consumed_auth_ticket_nonces = ConsumedAuthTicketNonces::<T>::get();
            let active_authentications = ActiveAuthentications::<T>::get();

            validate_authentication_attempt::<T>(
                &consumed_auth_ticket_nonces,
                &active_authentications,
                &auth_ticket,
            )
            .map_err(|err| {
                error!(message = "Authentication attemption failed", error = ?err);

                TransactionValidityError::Invalid(match err {
                    AuthenticationAttemptValidationError::NonceConflict => {
                        // The transaction renders nonce conflict if we have already seen this nonce
                        // before, so in practice, we will most likely observe this with auth ticket
                        // replays. We can sort of say the auth ticket is stale if it has already
                        // been consumed.
                        InvalidTransaction::Stale
                    }
                    AuthenticationAttemptValidationError::AlreadyAuthenticated => {
                        // Technically, we can't know if the transaction is from the future, but we
                        // know for sure it's not a replay, since the nonce didn't conflict;
                        // The way it usually observed to happen is when someone authenticates
                        // again while already having an active authentication - so this means this
                        // transaction would've been valid if sent in the future.
                        InvalidTransaction::Future
                    }
                })
            })?;

            // We must use non-default [`TransactionValidity`] here.
            ValidTransaction::with_tag_prefix("bioauth")
                .and_provides(auth_ticket)
                .priority(50)
                .longevity(1)
                .propagate(true)
                .build()
        }

        fn map_active_authentications_to_validators_set(
            active_authentications: &[Authentication<T::ValidatorPublicKey, T::Moment>],
        ) -> impl Iterator<Item = &T::ValidatorPublicKey> {
            active_authentications
                .iter()
                .map(|active_authentication| &active_authentication.public_key)
        }

        fn issue_validators_set_update(
            active_authentications: &[Authentication<T::ValidatorPublicKey, T::Moment>],
        ) {
            let validator_public_keys =
                Self::map_active_authentications_to_validators_set(active_authentications);
            T::ValidatorSetUpdater::update_validators_set(validator_public_keys);
        }

        fn issue_validators_set_init(
            active_authentications: &[Authentication<T::ValidatorPublicKey, T::Moment>],
        ) {
            let validator_public_keys =
                Self::map_active_authentications_to_validators_set(active_authentications);
            T::ValidatorSetUpdater::init_validators_set(validator_public_keys);
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
#[derive(Encode, Decode, Clone, Eq, PartialEq, Default, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CheckBioauthTx<T: Config + Send + Sync>(PhantomData<T>);

impl<T: Config + Send + Sync> CheckBioauthTx<T> {
    /// Creates new `SignedExtension` to check bioauth extrinsic.
    pub fn new() -> Self {
        Self(sp_std::marker::PhantomData)
    }
}

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

    fn pre_dispatch(
        self,
        who: &T::AccountId,
        call: &T::Call,
        info: &DispatchInfoOf<T::Call>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        self.validate(who, call, info, len).map(|_| ())
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
