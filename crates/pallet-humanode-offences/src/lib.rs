//! A substrate pallet containing the humanode offences handler logic.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    sp_std::prelude::*,
    traits::{Get, StorageVersion},
};
pub use pallet::*;
use sp_staking::offence::{Kind, Offence, OffenceError, ReportOffence};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

/// The offender type alias.
pub(crate) type OffenderOf<T> = pallet_humanode_session::IdentificationTupleFor<T>;

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;

    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_humanode_session::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Deauthentication reason on offence report.
        type DeauthenticationReasonOnOffenceReport: Get<Self::DeauthenticationReason>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// The total number of offences.
    #[pallet::storage]
    pub type Total<T: Config> = StorageValue<_, u64>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An event on reported offence.
        ReportedOffence {
            /// The offence kind.
            kind: Kind,
            /// The offenders list in report.
            offenders: Vec<OffenderOf<T>>,
        },
    }
}

impl<T: Config, O> ReportOffence<T::AccountId, OffenderOf<T>, O> for Pallet<T>
where
    O: Offence<OffenderOf<T>>,
{
    fn report_offence(_reporters: Vec<T::AccountId>, offence: O) -> Result<(), OffenceError> {
        let offenders = offence.offenders();

        Self::deposit_event(Event::ReportedOffence {
            kind: O::ID,
            offenders: offenders.clone(),
        });

        let mut maybe_should_be_deauthenticated = Vec::with_capacity(offenders.len());

        for offender in offenders {
            let (_offender, identity) = &offender;
            match identity {
                pallet_humanode_session::Identification::Bioauth(authentication) => {
                    maybe_should_be_deauthenticated.push(authentication.clone());
                }
                pallet_humanode_session::Identification::Bootnode(..) => {
                    // Never slash the bootnodes.
                }
            }
        }

        if !maybe_should_be_deauthenticated.is_empty() {
            let deauthenticated_number = <pallet_bioauth::Pallet<T>>::deauthenticate(
                maybe_should_be_deauthenticated,
                T::DeauthenticationReasonOnOffenceReport::get(),
            )
            .len();

            if deauthenticated_number != 0 {
                let offences_number: u64 = deauthenticated_number
                    .try_into()
                    .expect("u64 is big enough for this overflow to be practically impossible");

                <Total<T>>::mutate(|total| {
                    let new_total = total.map_or(offences_number, |t| {
                        t.checked_add(offences_number).expect(
                            "u64 is big enough for this overflow to be practically impossible",
                        )
                    });
                    *total = Some(new_total);
                });
            }
        }

        Ok(())
    }

    fn is_known_offence(_offenders: &[OffenderOf<T>], _time_slot: &O::TimeSlot) -> bool {
        false
    }
}
