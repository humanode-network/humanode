use jsonrpsee::types::ErrorObject;

pub fn to_json(error: impl Into<jsonrpsee::core::Error>) -> String {
    let error: jsonrpsee::core::Error = error.into();
    let error: ErrorObject = error.into();
    serde_json::to_string(&error).unwrap()
}

pub fn open_chain() -> biometrics_api_client::ErrorChain {
    biometrics_api_client::ErrorChain::Open {
        response_blob: "response blob".to_owned(),
        chain: biometrics_api_client::StepChainLink {
            nonce: 7,
            ciphertext: Box::new([1, 2, 3]),
        },
    }
}
