//! Redirect helpers for common UI flows.
use axum::{
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::{headers, routing::TypedPath};

use crate::{QueryParamState, utils::query_param_state::is_local_path};

/// Helper function to create a redirect response with a success alert query parameter.
pub fn redirect_success(path: impl TypedPath) -> Response {
    Redirect::to(
        &path
            .with_query_params(QueryParamState::success())
            .to_string(),
    )
    .into_response()
}

/// Redirect back to the page a form was submitted from, named by its `Referer`.
///
/// Only the referrer's path and query survive, and only as a safe local path: a
/// `Referer` is client-chosen, so echoing it into `Location` unchecked is an open
/// redirect. Anything else falls back to `default`.
pub fn redirect_to_referer(
    referer: &headers::Referer,
    default: impl std::fmt::Display,
) -> Redirect {
    match local_path_from_referer(referer) {
        Some(path) => Redirect::to(&path),
        None => Redirect::to(&default.to_string()),
    }
}

/// The referrer's path and query, if they form a safe local path.
fn local_path_from_referer(referer: &headers::Referer) -> Option<String> {
    let uri: Uri = referer.to_string().parse().ok()?;
    let path = uri.path_and_query()?.as_str();

    is_local_path(path).then(|| path.to_string())
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderValue, header::LOCATION};
    use axum_extra::headers::{Header, Referer};

    use super::*;

    fn referer(value: &str) -> Referer {
        let header = HeaderValue::from_str(value).expect("valid header value");
        Referer::decode(&mut [&header].into_iter()).expect("valid referer")
    }

    fn location(redirect: Redirect) -> String {
        redirect
            .into_response()
            .headers()
            .get(LOCATION)
            .expect("location header")
            .to_str()
            .expect("ascii location")
            .to_string()
    }

    /// A same-origin referrer sends the user back to where they came from.
    #[test]
    fn keeps_path_and_query_of_a_local_referer() {
        assert_eq!(
            location(redirect_to_referer(
                &referer("https://example.com/candidate-lists?highlight=1"),
                "/fallback"
            )),
            "/candidate-lists?highlight=1"
        );
    }

    /// Only the path survives, so a foreign host can never become the target.
    #[test]
    fn never_redirects_off_site() {
        for evil in [
            "https://evil.example",
            "https://evil.example/",
            "//evil.example/path",
        ] {
            let redirect = redirect_to_referer(&referer(evil), "/fallback");
            assert!(
                location(redirect).starts_with('/'),
                "{evil:?} must stay on this origin"
            );
        }
    }

    /// A referrer that yields no local path falls back to the default target.
    #[test]
    fn falls_back_when_referer_has_no_local_path() {
        for unusable in ["evil.example", "mailto:someone@example.com"] {
            assert_eq!(
                location(redirect_to_referer(&referer(unusable), "/fallback")),
                "/fallback",
                "{unusable:?} must fall back"
            );
        }
    }
}
