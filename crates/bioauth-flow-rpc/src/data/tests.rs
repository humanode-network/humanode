use super::*;

#[test]
fn step_v1_request_deserialization_first_step() {
    let sample = serde_json::json!({ "requestBlob": "request blob" });

    let request: StepV1Request = serde_json::from_value(sample).unwrap();

    assert_eq!(
        request,
        StepV1Request {
            request_blob: "request blob".to_owned(),
            context: None,
        }
    );
}

#[test]
fn step_v1_request_deserialization_next_step() {
    let sample = serde_json::json!({ "requestBlob": "request blob", "context": "0x010203" });

    let request: StepV1Request = serde_json::from_value(sample).unwrap();

    assert_eq!(
        request,
        StepV1Request {
            request_blob: "request blob".to_owned(),
            context: Some(vec![1, 2, 3].into()),
        }
    );
}

#[test]
fn enroll_step_v1_response_serialization_continue() {
    let response = EnrollStepV1Response {
        response_blob: "response blob".to_owned(),
        status: StepV1Status::Continue {
            context: vec![1, 2, 3].into(),
        },
    };

    assert_eq!(
        serde_json::to_value(response).unwrap(),
        serde_json::json!({
            "responseBlob": "response blob",
            "status": { "continue": { "context": "0x010203" } },
        })
    );
}

#[test]
fn enroll_step_v1_response_serialization_done() {
    let response = EnrollStepV1Response {
        response_blob: "response blob".to_owned(),
        status: StepV1Status::Done(()),
    };

    assert_eq!(
        serde_json::to_value(response).unwrap(),
        serde_json::json!({
            "responseBlob": "response blob",
            "status": { "done": null },
        })
    );
}

#[test]
fn authenticate_step_v1_response_serialization_done() {
    let response = AuthenticateStepV1Response {
        response_blob: "response blob".to_owned(),
        status: StepV1Status::Done(AuthenticateStepV1Done {
            auth_ticket: vec![1, 2, 3].into(),
            auth_ticket_signature: vec![4, 5, 6].into(),
        }),
    };

    assert_eq!(
        serde_json::to_value(response).unwrap(),
        serde_json::json!({
            "responseBlob": "response blob",
            "status": {
                "done": {
                    "authTicket": "0x010203",
                    "authTicketSignature": "0x040506",
                }
            },
        })
    );
}
