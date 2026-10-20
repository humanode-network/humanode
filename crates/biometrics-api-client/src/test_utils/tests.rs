use super::*;

#[test]
fn evals_properly() {
    assert_eq!(
        mkerr("MY_ERR_CODE", None).to_string(),
        serde_json::json!({ "errorCode": "MY_ERR_CODE" }).to_string()
    );

    assert_eq!(
        mkerr("MY_ERR_CODE", Some(sample_error_chain())).to_string(),
        serde_json::json!({
            "errorCode": "MY_ERR_CODE",
            "responseBlob": "response blob",
            "chain": { "nonce": 7, "ciphertext": "0x010203" },
        })
        .to_string()
    );
}
