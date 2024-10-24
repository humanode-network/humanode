//! A substrate pallet containing the humanode offences handler logic.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{sp_std::prelude::*, traits::StorageVersion};
pub use pallet::*;
use sp_staking::offence::{Offence, OffenceError, ReportOffence};

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

/// Interface that defines on offence handler implementation.
pub trait OnOffence<Offender> {
    /// Execute on offence logic for provided offenders.
    fn on_offence(offenders: Vec<Offender>);
}

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Offender type.
        type Offender;

        /// On offence handler.
        type OnOffence: OnOffence<Self::Offender>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);
}

impl<T: Config, O> ReportOffence<T::AccountId, T::Offender, O> for Pallet<T>
where
    O: Offence<T::Offender>,
{
    fn report_offence(_reporters: Vec<T::AccountId>, offence: O) -> Result<(), OffenceError> {
        let offenders = offence.offenders();

        T::OnOffence::on_offence(offenders);

        Ok(())
    }

    fn is_known_offence(_offenders: &[T::Offender], _time_slot: &O::TimeSlot) -> bool {
        false
    }
}
