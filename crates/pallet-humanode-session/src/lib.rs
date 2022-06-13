//! A substrate pallet containing the humanode session management logic.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_runtime::traits::Convert;
use sp_std::prelude::*;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;

    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_bioauth::Config + pallet_bootnodes::Config
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

        /// The max total amount of session validators.
        ///
        /// This should be an upper bound to fit both the bootnodes, and the bioauth validators.
        type MaxSessionValidators: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// A mapping between the [`T::AccountId`], and the [`IdentificationFor<T>`] for the current
    /// session.
    #[pallet::storage]
    pub type CurrentSessionIdentities<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, IdentificationFor<T>, OptionQuery>;
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

/// The identifcation type for a given config.
pub type IdentificationFor<T> = Identification<BootnodeIdFor<T>, BioauthAuthenticationFor<T>>;

/// The identifcation tuple type for a given config.
pub type IdentificationTupleFor<T> = (<T as frame_system::Config>::AccountId, IdentificationFor<T>);

impl<T: Config> Pallet<T> {
    /// Compute the list of the authorities, for use at new session planning.
    fn next_authorities() -> impl Iterator<Item = IdentificationTupleFor<T>> {
        let bootnodes = <pallet_bootnodes::Pallet<T>>::bootnodes()
            .into_iter()
            .map(|id| {
                (
                    T::BootnodeIdOf::convert(id.clone()),
                    Identification::Bootnode(id),
                )
            });

        let bioauth_active_authentications = <pallet_bioauth::Pallet<T>>::active_authentications()
            .into_inner()
            .into_iter()
            .filter_map(|authentication| {
                T::ValidatorPublicKeyOf::convert(authentication.public_key.clone())
                    .map(|account_id| (account_id, Identification::Bioauth(authentication)))
            });

        bootnodes.chain(bioauth_active_authentications)
    }

    /// Clears and re-populates the [`CurrentSessionIdentities`] with the entries.
    fn update_current_session_identities<'a>(
        new_entries: impl Iterator<Item = &'a IdentificationTupleFor<T>> + 'a,
    ) {
        let mut res = <CurrentSessionIdentities<T>>::clear(16, None);
        while let Some(cursor) = res.maybe_cursor {
            res = <CurrentSessionIdentities<T>>::clear(16, Some(&cursor));
        }

        for (account_id, identity) in new_entries {
            <CurrentSessionIdentities<T>>::insert(account_id, identity);
        }
    }
}

impl<T: Config> pallet_session::historical::SessionManager<T::AccountId, IdentificationFor<T>>
    for Pallet<T>
{
    fn new_session(_new_index: u32) -> Option<Vec<IdentificationTupleFor<T>>> {
        // Compute the next list of the authorities.
        let next_authorities = Self::next_authorities().collect::<Vec<_>>();

        // Set the list of authorities for the current session.
        Self::update_current_session_identities(next_authorities.iter());

        Some(next_authorities)
    }

    // This part of code is reachable, but we leave it empty
    // as we don't have any meaningful things to do here.
    fn start_session(_start_index: u32) {}
    fn end_session(_end_index: u32) {}
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

/// A converter that uses stored [`CurrentSessionIdentities`] mapping to provide an identification
/// for a given account.
pub struct CurrentSessionIdentificationOf<T>(
    core::marker::PhantomData<T>,
    core::convert::Infallible,
);

impl<T: Config> sp_runtime::traits::Convert<T::AccountId, Option<IdentificationFor<T>>>
    for CurrentSessionIdentificationOf<T>
{
    fn convert(account_id: T::AccountId) -> Option<IdentificationFor<T>> {
        <CurrentSessionIdentities<T>>::get(account_id)
    }
}
