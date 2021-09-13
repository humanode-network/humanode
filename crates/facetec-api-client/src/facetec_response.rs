//! Generic Facetec response deserializer.

use crate::ServerError;
use serde::{Deserialize, Deserializer};

/// The generic response.
#[derive(Debug)]
pub(crate) struct FacetecResponse<T>(Result<T, ServerError>);

impl<T> FacetecResponse<T> {
    /// Inner value of response
    pub fn into_inner(self) -> Result<T, ServerError> {
        self.0
    }
}

impl<'de, T> Deserialize<'de> for FacetecResponse<T>
where
    T: Deserialize<'de> + std::fmt::Debug,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[allow(clippy::missing_docs_in_private_items)]
        #[derive(Deserialize, Debug)]
        struct Helper<T> {
            error: bool,
            #[serde(flatten)]
            server_error: Option<ServerError>,
            #[serde(flatten)]
            content: Option<T>,
        }

        let helper = Helper::deserialize(deserializer)?;

        match helper {
            Helper {
                error: false,
                content: Some(content),
                ..
            } => Ok(FacetecResponse(Ok(content))),
            Helper {
                error: true,
                server_error: Some(server_error),
                ..
            } => Ok(FacetecResponse(Err(server_error))),
            helper => Err(serde::de::Error::custom(format!(
                "unable to pick variant: {:?}",
                helper
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_response_deserialization() {
        let sample_response = serde_json::json!({
            "error": true,
            "errorMessage": "No entry found in the database.",
            "success": false
        });

        let response: FacetecResponse<()> = serde_json::from_value(sample_response).unwrap();
        let expected_error = "No entry found in the database.".to_owned();

        assert_matches!(
            response,
            FacetecResponse(Err(ServerError {error_message})) if error_message == expected_error
        );
    }

    #[test]
    fn unexpected_error_in_success_response_deserialization() {
        let sample_response = serde_json::json!({
            "errorMessage": "Tried to search a groupName when that groupName does not exist. groupName: humanode. Try adding a 3D FaceMap by calling /3d-db/enroll first.",
            "errorToString": "java.lang.Exception: Tried to search a groupName when that groupName does not exist. groupName: humanode. Try adding a 3D FaceMap by calling /3d-db/enroll first.",
            "stackTrace": "java.lang.Exception: Tried to search a groupName when that groupName does not exist. groupName: humanode. Try adding a 3D FaceMap by calling /3d-db/enroll first.\\n\\tat com.facetec.standardserver.search.SearchManager.search(SearchManager.java:64)\\n\\tat com.facetec.standardserver.processors.SearchProcessor.processRequest(SearchProcessor.java:35)\\n\\tat com.facetec.standardserver.processors.CommonProcessor.handle(CommonProcessor.java:58)\\n\\tat com.sun.net.httpserver.Filter$Chain.doFilter(Filter.java:79)\\n\\tat sun.net.httpserver.AuthFilter.doFilter(AuthFilter.java:83)\\n\\tat com.sun.net.httpserver.Filter$Chain.doFilter(Filter.java:82)\\n\\tat sun.net.httpserver.ServerImpl$Exchange$LinkHandler.handle(ServerImpl.java:675)\\n\\tat com.sun.net.httpserver.Filter$Chain.doFilter(Filter.java:79)\\n\\tat sun.net.httpserver.ServerImpl$Exchange.run(ServerImpl.java:647)\\n\\tat java.util.concurrent.ThreadPoolExecutor.runWorker(ThreadPoolExecutor.java:1149)\\n\\tat java.util.concurrent.ThreadPoolExecutor$Worker.run(ThreadPoolExecutor.java:624)\\n\\tat java.lang.Thread.run(Thread.java:748)\\n",
            "success": false,
            "wasProcessed": true,
            "error": true,
            "serverInfo": {
                "version": "9.3.0",
                "type": "Standard",
                "mode": "Development Only",
                "notice": "You should only be reading this if you are in server-side code.  Please make sure you do not allow the FaceTec Server to be called from the public internet."
            }
        });

        let response: FacetecResponse<()> = serde_json::from_value(sample_response).unwrap();
        let expected_error = "Tried to search a groupName when that groupName does not exist. groupName: humanode. Try adding a 3D FaceMap by calling /3d-db/enroll first.".to_owned();

        assert_matches!(
            response,
            FacetecResponse(Err(ServerError {error_message})) if error_message == expected_error
        );
    }

    #[test]
    fn already_enrolled_response_deserialization() {
        let sample_response = serde_json::json!({
            "error": true,
            "errorMessage": "An enrollment already exists for this externalDatabaseRefID.",
            "success": false,
            "serverInfo": {
                "version": "9.0.0-SNAPSHOT",
                "mode": "Development Only",
                "notice": "You should only be reading this if you are in server-side code.  Please make sure you do not allow the FaceTec Server to be called from the public internet."
            }
        });

        let response: FacetecResponse<()> = serde_json::from_value(sample_response).unwrap();
        let expected_error =
            "An enrollment already exists for this externalDatabaseRefID.".to_owned();

        assert_matches!(
            response,
            FacetecResponse(Err(ServerError {error_message})) if error_message == expected_error
        );
    }

    #[test]
    fn unable_to_pick_variant_missing_error_message() {
        let sample_response = serde_json::json!({
            "error": true,
            "success": false,
            "serverInfo": {
                "version": "9.0.0-SNAPSHOT",
                "mode": "Development Only",
                "notice": "You should only be reading this if you are in server-side code.  Please make sure you do not allow the FaceTec Server to be called from the public internet."
            }
        });

        let response =
            serde_json::from_value::<FacetecResponse<crate::db_enroll::Response>>(sample_response)
                .unwrap_err()
                .to_string();
        let expected_error =
            "unable to pick variant: Helper { error: true, server_error: None, content: Some(Response { success: false }) }".to_owned();

        assert_eq!(response, expected_error);
    }

    #[test]
    fn unable_to_pick_variant_excess_error_message() {
        let sample_response = serde_json::json!({
            "error": false,
            "errorMessage": "An enrollment already exists for this externalDatabaseRefID."
        });

        let response =
            serde_json::from_value::<FacetecResponse<crate::db_enroll::Response>>(sample_response)
                .unwrap_err()
                .to_string();
        let expected_error =
        "unable to pick variant: Helper { error: false, server_error: Some(ServerError { error_message: \"An enrollment already exists for this externalDatabaseRefID.\" }), content: None }".to_owned();

        assert_eq!(response, expected_error);
    }

    #[test]
    fn correct_response_with_excess_error_message() {
        use crate::db_enroll::Response;

        let sample_response = serde_json::json!({
            "error": false,
            "errorMessage": "An enrollment already exists for this externalDatabaseRefID.",
            "success": false,
            "serverInfo": {
                "version": "9.0.0-SNAPSHOT",
                "mode": "Development Only",
                "notice": "You should only be reading this if you are in server-side code.  Please make sure you do not allow the FaceTec Server to be called from the public internet."
            }
        });

        let response: FacetecResponse<Response> = serde_json::from_value(sample_response).unwrap();

        assert_matches!(
            response,
            FacetecResponse(Ok(Response{success})) if !success
        );
    }
}
