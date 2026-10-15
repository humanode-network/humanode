use super::*;

#[test]
fn request_serialization_start() {
    let expected_request = serde_json::json!({
        "requestBlob": "request blob",
        "chain": { "tag": "start", "publicKey": [1, 2, 3] },
        "signature": [4, 5, 6],
    });

    let actual_request = serde_json::to_value(StepRequest {
        request_blob: "request blob",
        chain: StepChain::Start {
            public_key: &[1, 2, 3],
        },
        signature: &[4, 5, 6],
    })
    .unwrap();

    assert_eq!(expected_request, actual_request);
}

#[test]
fn request_serialization_next() {
    let expected_request = serde_json::json!({
        "requestBlob": "request blob",
        "chain": { "tag": "next", "nonce": 7, "ciphertext": [1, 2, 3] },
        "signature": [4, 5, 6],
    });

    let actual_request = serde_json::to_value(StepRequest {
        request_blob: "request blob",
        chain: StepChain::Next {
            nonce: 7,
            ciphertext: &[1, 2, 3],
        },
        signature: &[4, 5, 6],
    })
    .unwrap();

    assert_eq!(expected_request, actual_request);
}

#[test]
fn signing_payload_encoding_start() {
    let payload = StepSigningPayload {
        request_blob: "ab",
        chain: &StepChain::Start {
            public_key: &[1, 2, 3],
        },
    };

    assert_eq!(
        payload.encode(),
        [
            // The request blob: the compact length and the bytes.
            8, b'a', b'b', //
            // The chain variant index.
            0, //
            // The public key: the compact length and the bytes.
            12, 1, 2, 3,
        ]
    );
}

#[test]
fn signing_payload_encoding_next() {
    let payload = StepSigningPayload {
        request_blob: "ab",
        chain: &StepChain::Next {
            nonce: 7,
            ciphertext: &[1, 2, 3],
        },
    };

    assert_eq!(
        payload.encode(),
        [
            // The request blob: the compact length and the bytes.
            8, b'a', b'b', //
            // The chain variant index.
            1, //
            // The nonce, little-endian.
            7, 0, 0, 0, 0, 0, 0, 0, //
            // The ciphertext: the compact length and the bytes.
            12, 1, 2, 3,
        ]
    );
}

#[test]
fn response_deserialization_continue() {
    let sample_response = serde_json::json!({
        "responseBlob": "response blob",
        "status": "continue",
        "chain": { "nonce": 7, "ciphertext": [1, 2, 3] },
    });

    let response: StepResponse<()> = serde_json::from_value(sample_response).unwrap();

    assert_eq!(
        response,
        StepResponse {
            response_blob: "response blob".to_owned(),
            status: StepStatus::Continue {
                chain: StepChainLink {
                    nonce: 7,
                    ciphertext: Box::new([1, 2, 3]),
                },
            },
        }
    );
}
