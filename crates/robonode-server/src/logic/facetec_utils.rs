//! FaceTec utilities.

use facetec_api_client as ft;

/// An enum with all of the meaningful outcomes from the db search result.
pub enum DbSearchResult {
    /// A usual response.
    Response(ft::db_search::Response),
    /// A special case - an error indicating that the group we searched at doesn't exist.
    /// We can treat it as a valid response with no results for our use case.
    NoGroupError,
    /// Some other error occurred.
    OtherError,
}

/// An adapter of the db search results to better fit our logic.
pub fn db_search_result_adapter(
    search_res: Result<ft::db_search::Response, ft::Error>,
) -> DbSearchResult {
    match search_res {
        Ok(res) => DbSearchResult::Response(res),
        Err(ft::Error::Server(ft::ServerError { error_message }))
            if error_message
                .starts_with("Tried to search a groupName when that groupName does not exist.") =>
        {
            DbSearchResult::NoGroupError
        }
        Err(_) => DbSearchResult::OtherError,
    }
}
