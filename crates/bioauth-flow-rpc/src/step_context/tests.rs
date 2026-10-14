use super::*;

fn sample() -> StepContext {
    StepContext {
        nonce: 7,
        ciphertext: vec![1, 2, 3],
    }
}

#[test]
fn encoding() {
    assert_eq!(
        sample().to_bytes().0,
        [
            // The nonce, little-endian.
            7, 0, 0, 0, 0, 0, 0, 0, //
            // The ciphertext: the compact length and the bytes.
            12, 1, 2, 3,
        ]
    );
}

#[test]
fn round_trip() {
    let decoded = StepContext::from_bytes(&sample().to_bytes()).unwrap();
    assert_eq!(decoded, sample());
}

#[test]
fn rejects_truncated() {
    let mut bytes = sample().to_bytes();
    bytes.0.pop();
    StepContext::from_bytes(&bytes).unwrap_err();
}

#[test]
fn rejects_trailing() {
    let mut bytes = sample().to_bytes();
    bytes.0.push(0);
    StepContext::from_bytes(&bytes).unwrap_err();
}

#[test]
fn serializes_as_hex() {
    let json = serde_json::to_value(sample().to_bytes()).unwrap();
    assert_eq!(json, serde_json::json!("0x07000000000000000c010203"));
}
