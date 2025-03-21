//! A substrate pallet containing the humanode session management logic.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "try-runtime")]
use frame_support::sp_runtime::TryRuntimeError;
use frame_support::traits::{Get, StorageVersion};
pub use pallet::*;
use sp_runtime::traits::Convert;
use sp_std::prelude::*;
pub use weights::*;

pub mod weights;

mod migrations;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// The type representing the session index in our chain.
type SessionIndex = u32;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::BoundedBTreeSet;

    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config:
        frame_system::Config
        /* Session pallet is only required for migration to v1. */
        + pallet_session::Config
        + pallet_bioauth::Config
        + pallet_bootnodes::Config
    {
        /// The type for converting the key that `pallet_bioauth` uses into the key that session
        /// requires.
        /// Typically it will be a no-op, or an identity converter.
        type ValidatorPublicKeyOf: Convert<
            <Self as pallet_bioauth::Config>::ValidatorPublicKey,
            Option<Self::AccountId>,
        >;

        /// The type for converting the key that bootnodes use into the key that session requires.
        type BootnodeIdOf: Convert<<Self as pallet_bootnodes::Config>::BootnodeId, Self::AccountId>;

        /// The max amount of bootnodes contributing to the session validators.
        type MaxBootnodeValidators: Get<u32>;

        /// The max amount of bioauth-powered session validators.
        type MaxBioauthValidators: Get<u32>;

        /// The maximum number of banned accounts.
        type MaxBannedAccounts: Get<u32>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// A mapping between the session and the `T::AccountId` to the [`IdentificationFor<T>`].
    #[pallet::storage]
    pub type SessionIdentities<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        SessionIndex,
        Twox64Concat,
        T::AccountId,
        IdentificationFor<T>,
        OptionQuery,
    >;

    /// The number of the current session, as filled in by the session manager.
    #[pallet::storage]
    pub type CurrentSessionIndex<T: Config> = StorageValue<_, SessionIndex, OptionQuery>;

    /// A set of all banned accounts that can't be validators in the network.
    #[pallet::storage]
    pub type BannedAccounts<T: Config> =
        StorageValue<_, BoundedBTreeSet<T::AccountId, T::MaxBannedAccounts>, ValueQuery>;

    /// Possible errors.
    #[pallet::error]
    pub enum Error<T> {
        /// Banning bootnodes is not allowed.
        AttemptToBanBootnode,
        /// The account is already banned for ban call.
        AccountIsAlreadyBanned,
        /// The account is not banned for unban call.
        AccountIsNotBanned,
        /// The BannedAccounts storage has reached the limit as BoundedVec.
        TooManyBannedAccounts,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Ban account.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::ban(
            <BannedAccounts<T>>::get().len().try_into()
            .expect("u32 is big enough for this overflow to be practically impossible")
        ))]
        pub fn ban(origin: OriginFor<T>, account_id: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;

            if <pallet_bootnodes::Pallet<T>>::bootnodes()
                .iter()
                .any(|bootnode| T::BootnodeIdOf::convert(bootnode.clone()) == account_id)
            {
                return Err(Error::<T>::AttemptToBanBootnode.into());
            }

            <BannedAccounts<T>>::try_mutate::<_, DispatchError, _>(move |banned_accounts| {
                if !banned_accounts
                    .try_insert(account_id)
                    .map_err(|_| Error::<T>::TooManyBannedAccounts)?
                {
                    return Err(Error::<T>::AccountIsAlreadyBanned.into());
                }

                Ok(())
            })?;

            Ok(())
        }

        /// Unban account.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::unban(
            <BannedAccounts<T>>::get().len().try_into()
            .expect("u32 is big enough for this overflow to be practically impossible")
        ))]
        pub fn unban(origin: OriginFor<T>, account_id: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;

            BannedAccounts::<T>::try_mutate::<_, DispatchError, _>(move |banned_accounts| {
                if !banned_accounts.remove(&account_id) {
                    return Err(Error::<T>::AccountIsNotBanned.into());
                }

                Ok(())
            })?;

            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            migrations::v1::migrate::<T>()
        }

        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
            Ok(migrations::v1::pre_migrate::<T>())
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
            migrations::v1::post_migrate::<T>(state);
            Ok(())
        }
    }
}

/// The identification type, to indicate where does a particular validator comes from, in a given
/// session.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    scale_info::TypeInfo,
    codec::MaxEncodedLen,
    codec::Encode,
    codec::Decode,
)]
pub enum Identification<Bootnode, Bioauth> {
    /// The validator is a bootnode.
    Bootnode(Bootnode),
    /// The validator is bioauthenticated.
    Bioauth(Bioauth),
}

/// The bioauth authentication type for a given config.
pub type BioauthAuthenticationFor<T> = pallet_bioauth::Authentication<
    <T as pallet_bioauth::Config>::ValidatorPublicKey,
    <T as pallet_bioauth::Config>::Moment,
>;

/// The bootnode id type for a given config.
pub type BootnodeIdFor<T> = <T as pallet_bootnodes::Config>::BootnodeId;

/// The identification type for a given config.
pub type IdentificationFor<T> = Identification<BootnodeIdFor<T>, BioauthAuthenticationFor<T>>;

/// The identification tuple type for a given config.
pub type IdentificationTupleFor<T> = (<T as frame_system::Config>::AccountId, IdentificationFor<T>);

impl<T: Config> Pallet<T> {
    /// Compute the list of the authorities, for use at new session planning.
    fn next_authorities() -> impl Iterator<Item = IdentificationTupleFor<T>> {
        let bootnodes = <pallet_bootnodes::Pallet<T>>::bootnodes()
            .into_iter()
            .take(T::MaxBootnodeValidators::get().try_into().unwrap())
            .map(|id| {
                (
                    T::BootnodeIdOf::convert(id.clone()),
                    Identification::Bootnode(id),
                )
            });

        let banned_accounts = <BannedAccounts<T>>::get();

        let bioauth_active_authentications = <pallet_bioauth::Pallet<T>>::active_authentications()
            .into_inner()
            .into_iter()
            .take(T::MaxBioauthValidators::get().try_into().unwrap())
            .filter_map(move |authentication| {
                T::ValidatorPublicKeyOf::convert(authentication.public_key.clone())
                    .filter(|account_id| !banned_accounts.contains(account_id))
                    .map(|account_id| (account_id, Identification::Bioauth(authentication)))
            });

        bootnodes.chain(bioauth_active_authentications)
    }

    /// Clears and re-populates the [`SessionIdentities`] for a given session with the entries.
    fn update_session_identities<'a>(
        session_index: u32,
        new_entries: impl Iterator<Item = &'a IdentificationTupleFor<T>> + 'a,
    ) {
        Self::clear_session_identities(session_index);

        for (account_id, identity) in new_entries {
            <SessionIdentities<T>>::insert(session_index, account_id, identity);
        }
    }

    /// Clears the [`SessionIdentities`] for a given session.
    fn clear_session_identities(session_index: u32) {
        // TODO(#388): switch to `clear_prefix` after the API is fixed.
        #[allow(deprecated)]
        <SessionIdentities<T>>::remove_prefix(session_index, None);
    }
}

impl<T: Config> pallet_session::historical::SessionManager<T::AccountId, IdentificationFor<T>>
    for Pallet<T>
{
    fn new_session(new_index: u32) -> Option<Vec<IdentificationTupleFor<T>>> {
        // Compute the next list of the authorities.
        let next_authorities = Self::next_authorities().collect::<Vec<_>>();

        // Set the list of authorities for the current session.
        Self::update_session_identities(new_index, next_authorities.iter());

        Some(next_authorities)
    }

    fn start_session(start_index: u32) {
        <CurrentSessionIndex<T>>::put(start_index);
    }

    fn end_session(end_index: u32) {
        Self::clear_session_identities(end_index);
        <CurrentSessionIndex<T>>::kill();
    }
}

// In fact, the [`pallet_session::historical::SessionManager`] should not require the
// [`pallet_session::SessionManager`] implementation - this is a substrate design error.
// So, we implement it as unreachable.
impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
    fn new_session(_new_index: u32) -> Option<Vec<T::AccountId>> {
        unreachable!()
    }

    fn end_session(_end_index: u32) {
        unreachable!()
    }

    fn start_session(_start_index: u32) {
        unreachable!()
    }
}

/// A converter that uses stored [`SessionIdentities`] mapping to provide an identification
/// for a given account.
pub struct CurrentSessionIdentificationOf<T>(
    core::marker::PhantomData<T>,
    core::convert::Infallible,
);

impl<T: Config> sp_runtime::traits::Convert<T::AccountId, Option<IdentificationFor<T>>>
    for CurrentSessionIdentificationOf<T>
{
    fn convert(account_id: T::AccountId) -> Option<IdentificationFor<T>> {
        let session_index = <CurrentSessionIndex<T>>::get()?;
        <SessionIdentities<T>>::get(session_index, account_id)
    }
}
