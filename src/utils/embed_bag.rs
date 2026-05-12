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
use serde::{Deserialize, Serialize};
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
    limit: Option<usize>,
}

/// Parse a boolean-ish query value. `false`, `0`, `no` (case-insensitive) are
/// false; any other value — or a missing param — is true. Matches the upstream
/// `bag-address-lookup` HTTP service.
fn parse_bool(value: &str) -> bool {
    !matches!(value.to_ascii_lowercase().as_str(), "false" | "0" | "no")
}

/// Fuzzy-match localities and municipalities against the `wp` query param.
///
/// Returns a JSON array of structured suggestion records matching the wire
/// format of the upstream `bag-address-lookup` service (see [`SuggestResponse`]).
/// An empty array is a successful response meaning no match. Missing `wp`
/// yields `400 Bad Request` with an `{"error": "missing wp"}` body.
async fn suggest(Query(params): Query<SuggestQuery>) -> impl IntoResponse {
    let Some(query) = params.wp else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "missing wp"})),
        );
    };

    let include_municipalities = params.municipalities.as_deref().is_none_or(parse_bool);
    let limit = params.limit.unwrap_or(DEFAULT_SUGGEST_LIMIT);

    let entries = DATABASE.suggest(
        &query,
        DEFAULT_SUGGEST_THRESHOLD,
        limit,
        include_municipalities,
    );
    let body: Vec<SuggestResponse<'_>> = entries.iter().map(SuggestResponse::from).collect();
    let Ok(body) = serde_json::to_value(body) else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "failed to serialize response"})),
        );
    };

    (StatusCode::OK, Json(body))
}

/// Wire format for a single `/suggest` result, matching the upstream
/// `bag-address-lookup` HTTP service. `SuggestEntry` itself does not
/// derive `Serialize`, so we project it into this borrowing view.
#[derive(Serialize)]
#[serde(untagged)]
enum SuggestResponse<'a> {
    Locality {
        wp: &'a str,
        wp_code: u16,
        gm: &'a str,
        gm_code: u16,
        pv: &'a str,
        unique: bool,
        had_suffix: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        alias: Option<&'a str>,
    },
    Municipality {
        gm: &'a str,
        gm_code: u16,
        pv: &'a str,
        unique: bool,
        had_suffix: bool,
    },
}

impl<'a> From<&'a SuggestEntry> for SuggestResponse<'a> {
    fn from(entry: &'a SuggestEntry) -> Self {
        match entry {
            SuggestEntry::Locality {
                wp,
                wp_code,
                gm,
                gm_code,
                pv,
                unique,
                had_suffix,
                alias,
            } => SuggestResponse::Locality {
                wp,
                wp_code: *wp_code,
                gm,
                gm_code: *gm_code,
                pv,
                unique: *unique,
                had_suffix: *had_suffix,
                alias: *alias,
            },
            SuggestEntry::Municipality {
                gm,
                gm_code,
                pv,
                unique,
                had_suffix,
            } => SuggestResponse::Municipality {
                gm,
                gm_code: *gm_code,
                pv,
                unique: *unique,
                had_suffix: *had_suffix,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    use crate::test_utils::response_body_string;

    fn locality(
        name: &str,
        pv: &str,
        had_suffix: bool,
        alias: Option<&'static str>,
    ) -> SuggestEntry {
        SuggestEntry::Locality {
            wp: name.to_string(),
            wp_code: 1,
            gm: name.to_string(),
            gm_code: 2,
            pv: pv.to_string(),
            unique: true,
            had_suffix,
            alias,
        }
    }

    fn municipality(name: &str, pv: &str, had_suffix: bool) -> SuggestEntry {
        SuggestEntry::Municipality {
            gm: name.to_string(),
            gm_code: 3,
            pv: pv.to_string(),
            unique: true,
            had_suffix,
        }
    }

    #[test]
    fn locality_serializes_full_fields_in_upstream_order() {
        let entry = locality("Amsterdam", "NH", false, None);
        let json = serde_json::to_string(&SuggestResponse::from(&entry)).expect("serialize");
        assert_eq!(
            json,
            r#"{"wp":"Amsterdam","wp_code":1,"gm":"Amsterdam","gm_code":2,"pv":"NH","unique":true,"had_suffix":false}"#
        );
    }

    #[test]
    fn locality_with_alias_includes_alias_field() {
        let entry = locality("Bolsward", "FR", false, Some("Boalsert"));
        let json = serde_json::to_string(&SuggestResponse::from(&entry)).expect("serialize");
        assert!(json.contains(r#""alias":"Boalsert""#));
    }

    #[test]
    fn locality_without_alias_omits_alias_field() {
        let entry = locality("Amsterdam", "NH", true, None);
        let json = serde_json::to_string(&SuggestResponse::from(&entry)).expect("serialize");
        assert!(!json.contains("alias"));
    }

    #[test]
    fn municipality_serializes_full_fields_in_upstream_order() {
        let entry = municipality("Utrecht", "UT", false);
        let json = serde_json::to_string(&SuggestResponse::from(&entry)).expect("serialize");
        assert_eq!(
            json,
            r#"{"gm":"Utrecht","gm_code":3,"pv":"UT","unique":true,"had_suffix":false}"#
        );
    }

    #[test]
    fn had_suffix_is_a_flag_not_a_baked_in_name() {
        // The upstream wire format exposes had_suffix as a boolean; the caller
        // (frontend) is responsible for rendering "(PV)" when set. The name
        // fields themselves must stay bare.
        let entry = locality("Loo", "GE", true, None);
        let json = serde_json::to_string(&SuggestResponse::from(&entry)).expect("serialize");
        assert!(json.contains(r#""wp":"Loo""#));
        assert!(json.contains(r#""had_suffix":true"#));
        assert!(!json.contains("Loo (GE)"));
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
