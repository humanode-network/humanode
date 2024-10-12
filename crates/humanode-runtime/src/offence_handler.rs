//! The offence handler implementation.

use core::marker::PhantomData;

use sp_staking::offence::{Offence, OffenceError, ReportOffence};

use crate::*;

/// The offender type alias.
type Offender = pallet_session::historical::IdentificationTuple<Runtime>;

/// The reporter type alias.
type Reporter = AccountId;

/// The offence handler.
pub struct OffenceHandler<T>(PhantomData<T>);

impl<T, O> ReportOffence<Reporter, Offender, O> for OffenceHandler<T>
where
    T: ReportOffence<Reporter, Offender, O>,
    O: Offence<Offender>,
{
    fn report_offence(reporters: Vec<Reporter>, offence: O) -> Result<(), OffenceError> {
        let offenders = offence.offenders();
        T::report_offence(reporters, offence)?;

        match <O as Offence<Offender>>::ID {
            <pallet_im_online::UnresponsivenessOffence<Offender> as Offence<Offender>>::ID
            | <pallet_babe::EquivocationOffence<Offender> as Offence<Offender>>::ID
            | <pallet_grandpa::EquivocationOffence<Offender> as Offence<Offender>>::ID => {
                let mut should_be_deauthenticated = Vec::with_capacity(offenders.len());

                for details in offenders {
                    let (_offender, identity) = &details;
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
                    let _ = Bioauth::deauthenticate(
                        should_be_deauthenticated,
                        DeauthenticationReason::Offence,
                    );
                }
            }
            _ => {
                // Ignore other cases.
            }
        }

        Ok(())
    }

    fn is_known_offence(
        offenders: &[pallet_session::historical::IdentificationTuple<Runtime>],
        time_slot: &O::TimeSlot,
    ) -> bool {
        T::is_known_offence(offenders, time_slot)
    }
}
