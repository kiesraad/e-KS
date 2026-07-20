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

/// Whether the request query marks the page as part of an already-open
/// overlay (`overlay=true`), which suppresses the overlay open animation.
pub fn overlay_active(parts: &Parts) -> bool {
    parts
        .uri
        .query()
        .is_some_and(|q| q.contains("overlay=true"))
}
