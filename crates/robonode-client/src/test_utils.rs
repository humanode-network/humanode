pub fn mkerr(error_code: &str) -> serde_json::Value {
    serde_json::json!({ "errorCode": error_code })
}

pub fn mkerr_containing_blob(error_code: &str, scan_result_blob: &str) -> serde_json::Value {
    serde_json::json!({ "errorCode": error_code, "scanResultBlob": scan_result_blob })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evals_properly() {
        assert_eq!(
            mkerr("MY_ERR_CODE").to_string(),
            serde_json::json!({ "errorCode": "MY_ERR_CODE" }).to_string()
        );

        assert_eq!(
            mkerr_containing_blob("MY_ERR_CODE", "scan result blob").to_string(),
            serde_json::json!({ "errorCode": "MY_ERR_CODE", "scanResultBlob": "scan result blob" })
                .to_string()
        );
    }
}
