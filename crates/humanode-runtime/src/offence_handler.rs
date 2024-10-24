//! The offence handler implementation.

use pallet_humanode_offences::OnOffence as OnOffenceT;

use crate::*;

/// The offender type alias.
type Offender = pallet_session::historical::IdentificationTuple<Runtime>;

/// On offence handler.
pub struct OnOffence;

impl OnOffenceT<Offender> for OnOffence {
    fn on_offence(offenders: Vec<Offender>) {
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
            let _ =
                Bioauth::deauthenticate(should_be_deauthenticated, DeauthenticationReason::Offence);
        }
    }
}
