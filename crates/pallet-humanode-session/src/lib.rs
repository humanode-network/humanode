//! A substrate pallet containing the bioauth session integration logic.

#![cfg_attr(not(feature = "std"), no_std)]
// Fix clippy for sp_api::decl_runtime_apis!
#![allow(clippy::too_many_arguments, clippy::unnecessary_mut_passed)]

pub use pallet::*;
use sp_runtime::traits::Convert;
use sp_std::prelude::*;

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

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_bioauth::Config + pallet_bootnodes::Config
    {
        type ValidatorPublicKeyOf: Convert<
            <Self as pallet_bioauth::Config>::ValidatorPublicKey,
            Option<Self::AccountId>,
        >;

        type BootnodeIdOf: Convert<<Self as pallet_bootnodes::Config>::BootnodeId, Self::AccountId>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);
}

/// Full bioauth authentication type.
type FullBioauthAuthentication<T> = pallet_bioauth::Authentication<
    <T as pallet_bioauth::Config>::ValidatorPublicKey,
    <T as pallet_bioauth::Config>::Moment,
>;

type BootnodeIdOf<T> = <T as pallet_bootnodes::Config>::BootnodeId;

/// The identification type, to indicate where does a particular validator comes from, in a given
/// session.
pub enum Identification<T: Config> {
    /// The validator is a bootnode.
    Bootnode(BootnodeIdOf<T>),
    /// The validator is bioauthenticated.
    Bioauth(FullBioauthAuthentication<T>),
}

impl<T: Config> pallet_session::historical::SessionManager<T::AccountId, Identification<T>>
    for Pallet<T>
{
    fn new_session(_new_index: u32) -> Option<Vec<(T::AccountId, Identification<T>)>> {
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

        let next_authorities_data = bootnodes
            .chain(bioauth_active_authentications)
            .collect::<Vec<_>>();

        Some(next_authorities_data)
    }

    // This part of code is reachable, but we leave it empty
    // as we don't have any meaningful things to do here.
    fn start_session(_start_index: u32) {}
    fn end_session(_end_index: u32) {}
}

// In fact, the pallet_session::historical::SessionManager should not require the
// pallet_session::SessionManager implementation - this is a substrate design error.
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
