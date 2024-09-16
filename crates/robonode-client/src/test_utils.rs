pub enum ResponseIncludesBlob {
    Yes,
    No,
}

pub fn mkerr(error_code: &str, maybe_scan_result_blob: Option<&str>) -> serde_json::Value {
    match maybe_scan_result_blob {
        None => serde_json::json!({ "errorCode": error_code}),
        Some(scan_result_blob) => {
            serde_json::json!({ "errorCode": error_code, "scanResultBlob": scan_result_blob })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evals_properly() {
        assert_eq!(
            mkerr("MY_ERR_CODE", None).to_string(),
            serde_json::json!({ "errorCode": "MY_ERR_CODE" }).to_string()
        );

        assert_eq!(
            mkerr("MY_ERR_CODE", Some("scan result blob")).to_string(),
            serde_json::json!({ "errorCode": "MY_ERR_CODE", "scanResultBlob": "scan result blob" })
                .to_string()
        );
    }
}
