use crate::{ErrorChain, StepChainLink};

pub fn sample_error_chain() -> ErrorChain {
    ErrorChain::Open {
        response_blob: "response blob".to_owned(),
        chain: StepChainLink {
            nonce: 7,
            ciphertext: Box::new([1, 2, 3]),
        },
    }
}

pub fn mkerr(error_code: &str, maybe_chain: Option<ErrorChain>) -> serde_json::Value {
    match maybe_chain {
        None | Some(ErrorChain::Closed) => serde_json::json!({ "errorCode": error_code }),
        Some(ErrorChain::Open {
            response_blob,
            chain,
        }) => serde_json::json!({
            "errorCode": error_code,
            "responseBlob": response_blob,
            "chain": chain,
        }),
    }
}

#[cfg(test)]
mod tests;
