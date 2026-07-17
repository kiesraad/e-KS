//! Template-context flags derived from the incoming request, shared by the
//! app and CSB context extractors.

use axum::http::request::Parts;

/// Whether the request query asks for the success alert (`success=true`).
pub fn success_alert_requested(parts: &Parts) -> bool {
    parts
        .uri
        .query()
        .is_some_and(|q| q.contains("success=true"))
}

/// Whether the request came from an overlay page (via the referrer header).
pub fn overlay_referrer(parts: &Parts) -> bool {
    parts
        .headers
        .get(axum::http::header::REFERER)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|url| url.contains("overlay=true"))
}
