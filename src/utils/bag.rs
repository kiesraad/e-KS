//! Embedded BAG address lookup: handles `/lookup` and `/suggest` in-process
//! using the `bag-address-lookup` library
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
/// embedded BAG database
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

/// Check whether a full address exists verbatim in the BAG.
///
/// The postal code and house number are looked up, and the result only counts
/// as a match when the resolved street and locality also equal the supplied
/// `street_name` and `locality`. Returns `false` when the postal code + house
/// number combination is unknown.
pub fn address_exists(
    postal_code: &str,
    house_number: u32,
    street_name: &str,
    locality: &str,
) -> bool {
    match DATABASE.lookup(postal_code, house_number) {
        Some((pr, wp)) => pr == street_name && wp == locality,
        None => false,
    }
}

#[derive(Default, Deserialize)]
struct SuggestQuery {
    wp: Option<String>,
    #[serde(default)]
    municipalities: bool,
    #[serde(default)]
    aliases: bool,
    limit: Option<usize>,
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

    let limit = params.limit.unwrap_or(DEFAULT_SUGGEST_LIMIT);

    let names = DATABASE.suggest(
        &query,
        DEFAULT_SUGGEST_THRESHOLD,
        limit,
        params.municipalities,
        params.aliases,
    );

    (StatusCode::OK, Json(json!(names)))
}

/// Check whether `locality` is an exact, known place name in the BAG.
///
/// This runs the same fuzzy `suggest` matching as the endpoint but only
/// reports `true` when a suggestion equals the input exactly, so a prefix of a
/// real locality (e.g. `Amsterda`) is rejected. `with_municipalities` also
/// considers municipality names; `with_aliases` also considers Frisian
/// locality aliases.
pub fn locality_exists(locality: &str, with_municipalities: bool, with_aliases: bool) -> bool {
    DATABASE
        .suggest(
            locality,
            DEFAULT_SUGGEST_THRESHOLD,
            1,
            with_municipalities,
            with_aliases,
        )
        .iter()
        .any(|name| name == locality)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::response_body_string;

    #[tokio::test]
    async fn lookup_unknown_postal_code_returns_not_found() {
        let response = lookup(Query(LookupQuery {
            pc: "0000ZZ".to_string(),
            n: 1,
        }))
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response_body_string(response).await;
        assert!(body.contains("address not found"));
    }

    #[tokio::test]
    async fn suggest_missing_wp_returns_bad_request() {
        let response = suggest(Query(SuggestQuery::default()))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response_body_string(response).await;
        assert!(body.contains("missing wp"));
    }

    #[tokio::test]
    async fn suggest_non_matching_query_returns_empty_array() {
        let response = suggest(Query(SuggestQuery {
            wp: Some("zzzqqqxxxnotacity".to_string()),
            ..Default::default()
        }))
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        // An empty array is a successful "no match" response, not an error.
        assert_eq!(body.trim(), "[]");
    }

    #[tokio::test]
    async fn suggest_returns_flat_array_of_names() {
        let response = suggest(Query(SuggestQuery {
            wp: Some("Amsterdam".to_string()),
            limit: Some(1),
            ..Default::default()
        }))
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        // The body is a flat JSON array of strings, not structured records.
        assert_eq!(body.trim(), r#"["Amsterdam"]"#);
    }

    #[test]
    fn locality_exists_matches_exact_locality_only() {
        assert!(locality_exists("Amsterdam", false, false));
        // A prefix of a real locality is not itself a locality.
        assert!(!locality_exists("Amsterda", false, false));
    }

    #[test]
    fn locality_exists_honours_municipalities_flag() {
        // "Land van Cuijk" is a municipality, only found when municipalities
        // are included.
        assert!(locality_exists("Land van Cuijk", true, false));
        assert!(!locality_exists("Land van Cuijk", false, false));
    }

    #[test]
    fn locality_exists_honours_aliases_flag() {
        // "Boelensloane" is a Frisian alias, only found when aliases are
        // included.
        assert!(locality_exists("Boelensloane", false, true));
        assert!(!locality_exists("Boelensloane", false, false));
    }
}
