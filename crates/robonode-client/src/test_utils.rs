pub fn mkerr(error_code: &str) -> serde_json::Value {
    serde_json::json!({ "errorCode": error_code })
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
    }
}
