//! Embedded BAG address lookup: handles `/lookup` and `/suggest` in-process
//! using the `bag-address-lookup` library, instead of proxying to an external
//! bag-service.
use std::sync::LazyLock;

use axum::{
    Router,
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
};
use bag_address_lookup::{DEFAULT_SUGGEST_LIMIT, DEFAULT_SUGGEST_THRESHOLD, DatabaseHandle};
use serde::Deserialize;
use serde_json::json;

/// Lazily-decoded handle to the BAG database embedded in `bag-address-lookup`.
///
/// Initialised on first `/lookup` or `/suggest` request and reused for the
/// lifetime of the process; decoding the compressed database takes noticeable
/// time, so the first request is slower than subsequent ones.
static DATABASE: LazyLock<DatabaseHandle> =
    LazyLock::new(|| DatabaseHandle::load().expect("failed to load embedded BAG database"));

/// Axum router exposing the `/lookup` and `/suggest` endpoints backed by the
/// embedded BAG database, drop-in compatible with the external bag-service
/// proxy the frontend otherwise talks to.
pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/lookup", get(lookup))
        .route("/suggest", get(suggest))
}

#[derive(Deserialize)]
struct LookupQuery {
    pc: String,
    n: u32,
}

/// Resolve a postal code + house number to its street and locality.
///
/// Returns `{"pr": <street>, "wp": <locality>}` on a match, `404` with an
/// `{"error": ...}` body when no address matches. Malformed query strings
/// (missing `pc`/`n`, non-numeric `n`) are rejected by the `Query` extractor
/// as `400 Bad Request` before the handler runs.
async fn lookup(Query(params): Query<LookupQuery>) -> impl IntoResponse {
    match DATABASE.lookup(&params.pc, params.n) {
        Some((pr, wp)) => (StatusCode::OK, Json(json!({"pr": pr, "wp": wp}))),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "address not found"})),
        ),
    }
}

#[derive(Deserialize)]
struct SuggestQuery {
    wp: Option<String>,
    municipalities: Option<String>,
    aliases: Option<String>,
    limit: Option<usize>,
}

/// Parse a boolean-ish query value. `false`, `0`, `no` (case-insensitive) are
/// false; any other value is true. Matches the upstream `bag-address-lookup`
/// HTTP service.
fn parse_bool(value: &str) -> bool {
    !matches!(value.to_ascii_lowercase().as_str(), "false" | "0" | "no")
}

/// Fuzzy-match localities and municipalities against the `wp` query param.
///
/// Returns a JSON array of suggestion names, matching the wire format of the
/// upstream `bag-address-lookup` service. A name already carries a province
/// suffix (e.g. `Bergen (LI)`) when the source disambiguated a duplicate place
/// name. An empty array is a successful response meaning no match; missing
/// `wp` yields `400 Bad Request` with an `{"error": "missing wp"}` body.
///
/// `municipalities` (default true) toggles whether municipality names are
/// offered; `aliases` (default false) toggles whether Frisian locality aliases
/// are offered as suggestions in their own right.
async fn suggest(Query(params): Query<SuggestQuery>) -> impl IntoResponse {
    let Some(query) = params.wp else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "missing wp"})),
        );
    };

    let include_municipalities = params.municipalities.as_deref().is_none_or(parse_bool);
    let include_aliases = params.aliases.as_deref().is_some_and(parse_bool);
    let limit = params.limit.unwrap_or(DEFAULT_SUGGEST_LIMIT);

    let names = DATABASE.suggest(
        &query,
        DEFAULT_SUGGEST_THRESHOLD,
        limit,
        include_municipalities,
        include_aliases,
    );

    (StatusCode::OK, Json(json!(names)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    use crate::test_utils::response_body_string;

    #[tokio::test]
    async fn suggest_missing_wp_returns_bad_request() {
        let app = router::<()>();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/suggest")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response_body_string(response).await;
        assert!(body.contains("missing wp"));
    }

    #[tokio::test]
    async fn lookup_missing_params_is_rejected_by_extractor() {
        let app = router::<()>();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/lookup")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn lookup_unknown_postal_code_returns_not_found() {
        let app = router::<()>();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/lookup?pc=0000ZZ&n=1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response_body_string(response).await;
        assert!(body.contains("address not found"));
    }

    #[tokio::test]
    async fn suggest_non_matching_query_returns_empty_array() {
        let app = router::<()>();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/suggest?wp=zzzqqqxxxnotacity")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert_eq!(body.trim(), "[]");
    }

    #[tokio::test]
    async fn suggest_returns_flat_array_of_names() {
        let app = router::<()>();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/suggest?wp=Amsterdam&limit=1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        // The body is a flat JSON array of strings, not structured records.
        assert_eq!(body.trim(), r#"["Amsterdam"]"#);
    }
}
