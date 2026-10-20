//! The `authenticate_step_v1` method error.

use crate::error;

/// The `authenticate_step_v1` method error kinds.
#[derive(Debug)]
pub struct Error(pub error::shared::StepFlowError<biometrics_api_client::AuthenticateStepError>);

impl From<Error> for jsonrpsee::core::Error {
    fn from(err: Error) -> Self {
        err.0.to_jsonrpsee_error(|call_error| match call_error {
            biometrics_api_client::AuthenticateStepError::PersonNotFound(chain)
            | biometrics_api_client::AuthenticateStepError::FaceScanRejected(chain)
            | biometrics_api_client::AuthenticateStepError::LogicInternal(chain)
            | biometrics_api_client::AuthenticateStepError::UnknownCode(_, chain) => {
                error::data::StepRetry::from_error_chain(chain)
            }

            biometrics_api_client::AuthenticateStepError::SignatureInvalid
            | biometrics_api_client::AuthenticateStepError::ChainInvalid
            | biometrics_api_client::AuthenticateStepError::ChainDone
            | biometrics_api_client::AuthenticateStepError::NonceInvalid
            | biometrics_api_client::AuthenticateStepError::Unknown(_) => None,
        })
    }
}

#[cfg(test)]
mod tests;
