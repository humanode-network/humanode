use super::*;
use crate::test_utils::{mkerr, sample_error_chain};

#[test]
fn decodes_closed() {
    let err = mkerr("MY_ERR_CODE", None).to_string();
    let ErrorResponse { error_code, chain } = err.try_into().unwrap();
    assert_eq!(error_code, "MY_ERR_CODE");
    assert_eq!(chain, ErrorChain::Closed);
}

#[test]
fn decodes_open() {
    let err = mkerr("MY_ERR_CODE", Some(sample_error_chain())).to_string();
    let ErrorResponse { error_code, chain } = err.try_into().unwrap();
    assert_eq!(error_code, "MY_ERR_CODE");
    assert_eq!(chain, sample_error_chain());
}

#[test]
fn rejects_blob_without_chain() {
    let err = serde_json::json!({ "errorCode": "MY_ERR_CODE", "responseBlob": "blob" }).to_string();
    let res: Result<ErrorResponse, _> = err.clone().try_into();
    assert_eq!(res.unwrap_err(), err);
}

#[test]
fn rejects_chain_without_blob() {
    let err = serde_json::json!({
        "errorCode": "MY_ERR_CODE",
        "chain": { "nonce": 7, "ciphertext": [1, 2, 3] },
    })
    .to_string();
    let res: Result<ErrorResponse, _> = err.clone().try_into();
    assert_eq!(res.unwrap_err(), err);
}
