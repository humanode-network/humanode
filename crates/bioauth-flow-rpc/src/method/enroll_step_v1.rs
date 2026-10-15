//! The `enroll_step_v1` method error.

use crate::error;

/// The `enroll_step_v1` method error kinds.
#[derive(Debug)]
pub struct Error(pub error::shared::StepFlowError<biometrics_api_client::EnrollStepError>);

impl From<Error> for jsonrpsee::core::Error {
    fn from(err: Error) -> Self {
        err.0.to_jsonrpsee_error(|call_error| match call_error {
            biometrics_api_client::EnrollStepError::InvalidPublicKey(chain)
            | biometrics_api_client::EnrollStepError::FaceScanRejected(chain)
            | biometrics_api_client::EnrollStepError::PublicKeyAlreadyUsed(chain)
            | biometrics_api_client::EnrollStepError::PersonAlreadyEnrolled(chain)
            | biometrics_api_client::EnrollStepError::LogicInternal(chain)
            | biometrics_api_client::EnrollStepError::UnknownCode(_, chain) => {
                error::data::StepRetry::from_error_chain(chain)
            }

            biometrics_api_client::EnrollStepError::SignatureInvalid
            | biometrics_api_client::EnrollStepError::ChainInvalid
            | biometrics_api_client::EnrollStepError::ChainDone
            | biometrics_api_client::EnrollStepError::NonceInvalid
            | biometrics_api_client::EnrollStepError::Unknown(_) => None,
        })
    }
}

#[cfg(test)]
mod tests;
