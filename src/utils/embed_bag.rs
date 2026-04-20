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
use bag_address_lookup::{
    DEFAULT_SUGGEST_LIMIT, DEFAULT_SUGGEST_THRESHOLD, DatabaseHandle, SuggestEntry,
};
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
}

/// Fuzzy-match localities and municipalities against the `wp` query param.
///
/// Returns a JSON array of display strings (see [`format_entry`]). An empty
/// array is a successful response meaning no match. Missing `wp` yields
/// `400 Bad Request` with an `{"error": "missing wp"}` body.
async fn suggest(Query(params): Query<SuggestQuery>) -> impl IntoResponse {
    let Some(query) = params.wp else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "missing wp"})),
        );
    };

    let entries = DATABASE.suggest(&query, DEFAULT_SUGGEST_THRESHOLD, DEFAULT_SUGGEST_LIMIT);
    let body: Vec<String> = entries.iter().map(format_entry).collect();
    let Ok(body) = serde_json::to_value(body) else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "failed to serialize response"})),
        );
    };

    (StatusCode::OK, Json(body))
}

/// Render a suggestion as the string shown to the user in the datalist.
///
/// Plain name by default; when the source BAG name carried a stripped province
/// suffix (`had_suffix`), the province is appended as `"Name (PV)"` to
/// disambiguate localities/municipalities that share a name across provinces.
fn format_entry(entry: &SuggestEntry) -> String {
    let (name, had_suffix, pv) = match entry {
        SuggestEntry::Locality {
            wp, pv, had_suffix, ..
        } => (wp, had_suffix, pv),
        SuggestEntry::Municipality {
            gm, pv, had_suffix, ..
        } => (gm, had_suffix, pv),
    };

    if *had_suffix {
        format!("{} ({})", name, pv)
    } else {
        name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    use crate::test_utils::response_body_string;

    fn locality(name: &str, pv: &str, had_suffix: bool) -> SuggestEntry {
        SuggestEntry::Locality {
            wp: name.to_string(),
            wp_code: 0,
            gm: name.to_string(),
            gm_code: 0,
            pv: pv.to_string(),
            unique: true,
            had_suffix,
        }
    }

    fn municipality(name: &str, pv: &str, had_suffix: bool) -> SuggestEntry {
        SuggestEntry::Municipality {
            gm: name.to_string(),
            gm_code: 0,
            pv: pv.to_string(),
            unique: true,
            had_suffix,
        }
    }

    #[test]
    fn format_entry_locality_without_suffix_uses_bare_name() {
        assert_eq!(
            format_entry(&locality("Amsterdam", "NH", false)),
            "Amsterdam"
        );
    }

    #[test]
    fn format_entry_locality_with_suffix_appends_province() {
        assert_eq!(format_entry(&locality("Loo", "GE", true)), "Loo (GE)");
    }

    #[test]
    fn format_entry_municipality_without_suffix_uses_bare_name() {
        assert_eq!(
            format_entry(&municipality("Utrecht", "UT", false)),
            "Utrecht"
        );
    }

    #[test]
    fn format_entry_municipality_with_suffix_appends_province() {
        assert_eq!(
            format_entry(&municipality("Hengelo", "OV", true)),
            "Hengelo (OV)"
        );
    }

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
}
