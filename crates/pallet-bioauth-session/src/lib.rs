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
    pub trait Config: frame_system::Config + pallet_bioauth::Config {
        type ValidatorPublicKeyOf: Convert<
            <Self as pallet_bioauth::Config>::ValidatorPublicKey,
            Option<Self::AccountId>,
        >;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);
}

/// Full bioauth authentication type.
type FullBioauthAuthentication<T> = pallet_bioauth::Authentication<
    <T as pallet_bioauth::Config>::ValidatorPublicKey,
    <T as pallet_bioauth::Config>::Moment,
>;

impl<T: Config>
    pallet_session::historical::SessionManager<
        T::AccountId,
        pallet_bioauth::Authentication<
            <T as pallet_bioauth::Config>::ValidatorPublicKey,
            <T as pallet_bioauth::Config>::Moment,
        >,
    > for Pallet<T>
{
    fn new_session(_new_index: u32) -> Option<Vec<(T::AccountId, FullBioauthAuthentication<T>)>> {
        let next_authorities_data = <pallet_bioauth::Pallet<T>>::active_authentications()
            .into_inner()
            .iter()
            .filter_map(|authentication| {
                T::ValidatorPublicKeyOf::convert(authentication.public_key.clone())
                    .map(|account_id| (account_id, authentication.clone()))
            })
            .collect::<Vec<_>>();

        Some(next_authorities_data)
    }

    // This part of code is reachable but we leave it empty
    // as we don't have any bioauth related logic code here.
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
