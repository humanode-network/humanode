use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Sample {
    #[serde(with = "crate::hex_bytes")]
    bytes: Box<[u8]>,
}

fn sample(bytes: impl Into<Box<[u8]>>) -> Sample {
    Sample {
        bytes: bytes.into(),
    }
}

#[test]
fn serializes_lowercase_with_prefix() {
    let json = serde_json::to_string(&sample([0xde, 0xad, 0xbe, 0xef])).unwrap();
    assert_eq!(json, r#"{"bytes":"0xdeadbeef"}"#);
}

#[test]
fn serializes_empty() {
    let json = serde_json::to_string(&sample([])).unwrap();
    assert_eq!(json, r#"{"bytes":"0x"}"#);
}

#[test]
fn deserializes_lowercase() {
    let value: Sample = serde_json::from_str(r#"{"bytes":"0xdeadbeef"}"#).unwrap();
    assert_eq!(value, sample([0xde, 0xad, 0xbe, 0xef]));
}

#[test]
fn deserializes_uppercase() {
    let value: Sample = serde_json::from_str(r#"{"bytes":"0xDEADBEEF"}"#).unwrap();
    assert_eq!(value, sample([0xde, 0xad, 0xbe, 0xef]));
}

#[test]
fn deserializes_mixed_case() {
    let value: Sample = serde_json::from_str(r#"{"bytes":"0xDeAdBeEf"}"#).unwrap();
    assert_eq!(value, sample([0xde, 0xad, 0xbe, 0xef]));
}

#[test]
fn deserializes_empty() {
    let value: Sample = serde_json::from_str(r#"{"bytes":"0x"}"#).unwrap();
    assert_eq!(value, sample([]));
}

#[test]
fn rejects_missing_prefix() {
    let err = serde_json::from_str::<Sample>(r#"{"bytes":"deadbeef"}"#).unwrap_err();
    assert!(
        err.to_string().starts_with("missing the 0x prefix"),
        "{err}"
    );
}

#[test]
fn rejects_uppercase_prefix() {
    let err = serde_json::from_str::<Sample>(r#"{"bytes":"0XDEADBEEF"}"#).unwrap_err();
    assert!(
        err.to_string().starts_with("missing the 0x prefix"),
        "{err}"
    );
}

#[test]
fn rejects_non_hex() {
    let err = serde_json::from_str::<Sample>(r#"{"bytes":"0xzz"}"#).unwrap_err();
    assert!(err.to_string().starts_with("Invalid character"), "{err}");
}

#[test]
fn rejects_odd_length() {
    let err = serde_json::from_str::<Sample>(r#"{"bytes":"0xabc"}"#).unwrap_err();
    assert!(err.to_string().starts_with("Odd number of digits"), "{err}");
}

#[test]
fn rejects_number_array() {
    let err = serde_json::from_str::<Sample>(r#"{"bytes":[1,2,3]}"#).unwrap_err();
    assert!(err.to_string().starts_with("invalid type"), "{err}");
}
