use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::routing::TypedPath;

use crate::QueryParamState;

/// Helper function to create a redirect response with a success alert query parameter.
pub fn redirect_success(path: impl TypedPath) -> Response {
    Redirect::to(
        &path
            .with_query_params(QueryParamState::success())
            .to_string(),
    )
    .into_response()
}
