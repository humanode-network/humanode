//! A substrate pallet containing the humanode offences handler logic.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    sp_std::prelude::*,
    traits::{Get, StorageVersion},
};
pub use pallet::*;
use sp_staking::offence::{Offence, OffenceError, ReportOffence};

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_humanode_session::Config {
        /// Deauthentication reason on offence report.
        type DeauthenticationReasonOnOffenceReport: Get<Self::DeauthenticationReason>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);
}

/// The offender type alias.
type OffenderOf<T> = pallet_humanode_session::IdentificationTupleFor<T>;

impl<T: Config, O> ReportOffence<T::AccountId, OffenderOf<T>, O> for Pallet<T>
where
    O: Offence<OffenderOf<T>>,
{
    fn report_offence(_reporters: Vec<T::AccountId>, offence: O) -> Result<(), OffenceError> {
        let offenders = offence.offenders();
        let mut should_be_deauthenticated = Vec::with_capacity(offenders.len());

        for offender in offenders {
            let (_offender, identity) = &offender;
            match identity {
                pallet_humanode_session::Identification::Bioauth(authentication) => {
                    should_be_deauthenticated.push(authentication.public_key.clone());
                }
                pallet_humanode_session::Identification::Bootnode(..) => {
                    // Never slash the bootnodes.
                }
            }
        }

        if !should_be_deauthenticated.is_empty() {
            let _ = <pallet_bioauth::Pallet<T>>::deauthenticate(
                should_be_deauthenticated,
                T::DeauthenticationReasonOnOffenceReport::get(),
            );
        }

        Ok(())
    }

    fn is_known_offence(_offenders: &[OffenderOf<T>], _time_slot: &O::TimeSlot) -> bool {
        false
    }
}
