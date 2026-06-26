//! Database maintenance gate.
//!
//! When the database is unavailable (see [`crate::DbHealth`]), every
//! DB-dependent route is short-circuited to a static 503 maintenance page
use crate::{AppState, Context, HtmlTemplate, Locale, filters};
use askama::Template;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode, Uri, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

#[derive(Template)]
#[template(path = "common/pages/maintenance.html")]
struct MaintenanceTemplate {
    retry_path: String,
}

struct MaintenanceValues {
    locale: Locale,
}

impl askama::Values for MaintenanceValues {
    fn get_value<'a>(&'a self, key: &str) -> Option<&'a dyn std::any::Any> {
        match key {
            "locale" => Some(&self.locale as &dyn std::any::Any),
            _ => None,
        }
    }
}

/// Paths that must keep working while the database is unavailable: the health
/// probe (so orchestration can still read liveness) and static assets / live
/// reload (so the maintenance page can load its stylesheet).
fn is_exempt(path: &str) -> bool {
    path == "/health"
        || path.starts_with("/static/")
        || path.starts_with("/.well-known/")
        || path.starts_with("/livereload/")
}

/// Middleware that serves the maintenance page for DB-dependent routes while
/// the database is marked unavailable.
pub async fn db_gate_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if state.db_health.is_healthy() || is_exempt(request.uri().path()) {
        return next.run(request).await;
    }

    maintenance_response(request.headers(), request.uri())
}

/// Render the static 503 maintenance page, localized from `Accept-Language`,
/// with a `Retry-After` header and a "try again" link back to `uri`.
pub fn maintenance_response(headers: &HeaderMap, uri: &Uri) -> Response {
    let locale = headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|value| value.to_str().ok())
        .and_then(Locale::from_accept_language)
        .unwrap_or_default();

    let retry_path = uri
        .path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| "/".to_string());

    let mut response = HtmlTemplate(
        MaintenanceTemplate { retry_path },
        MaintenanceValues { locale },
    )
    .into_response();

    *response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
    response
        .headers_mut()
        .insert(header::RETRY_AFTER, header::HeaderValue::from_static("30"));

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::response_body_string;

    #[test]
    fn exempts_health_static_and_wellknown() {
        assert!(is_exempt("/health"));
        assert!(is_exempt("/static/index.css"));
        assert!(is_exempt("/.well-known/security.txt"));
        assert!(!is_exempt("/"));
        assert!(!is_exempt("/login"));
    }

    #[tokio::test]
    async fn maintenance_response_is_503_with_retry_after() {
        let uri: Uri = "/persons?success=true".parse().unwrap();
        let response = maintenance_response(&HeaderMap::new(), &uri);

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.headers().get(header::RETRY_AFTER).unwrap(), "30");

        let body = response_body_string(response).await;
        // The retry link points back at the originating page.
        assert!(body.contains("href=\"/persons?success=true\""));
    }

    #[tokio::test]
    async fn maintenance_response_localizes_from_accept_language() {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT_LANGUAGE, "en-US,en;q=0.9".parse().unwrap());
        let response = maintenance_response(&headers, &Uri::from_static("/"));
        let body = response_body_string(response).await;
        assert!(body.contains("Temporarily unavailable"));
    }
}
