use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::routing::TypedPath;

pub const SUCCESS_ALERT_QUERY: [(&str, &str); 1] = [("alert", "success")];

/// Helper function to create a redirect response with a success alert query parameter.
pub fn redirect_success(path: impl TypedPath) -> Response {
    Redirect::to(&path.with_query_params(SUCCESS_ALERT_QUERY).to_string()).into_response()
}
