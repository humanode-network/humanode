//! Utilities for serde.

use crate::ServerError;
use serde::{Deserialize, Deserializer};

/// Internal type to parse values on the contents.
/// Useful for extracting errors from 200-ok responses.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
enum Either<A, B> {
    /// Left variant.
    Left(A),
    /// Right variant.
    Right(B),
}

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
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = Either::deserialize(deserializer)?;

        Ok(FacetecResponse(match helper {
            Either::Left(server_error) => Err(server_error),
            Either::Right(correct_response) => Ok(correct_response),
        }))
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
            FacetecResponse(Err(ServerError {error_message: err})) if err == expected_error
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
            FacetecResponse(Err(ServerError {error_message: err})) if err == expected_error
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
            FacetecResponse(Err(ServerError {error_message: err})) if err == expected_error
        );
    }
}
