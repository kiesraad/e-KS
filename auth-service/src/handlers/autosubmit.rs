use crate::bindings::http_post::AUTOSUBMIT_JS;
use axum::{
    http::header,
    response::{IntoResponse, Response},
};

/// Serve the small auto-submit script referenced by the HTTP-POST binding
/// page. Kept as an external resource so the binding page complies with a
/// strict CSP (no `unsafe-inline`).
pub async fn handle_autosubmit_js(_: crate::AutosubmitJsPath) -> Response {
    (
        [
            (
                header::CONTENT_TYPE,
                "application/javascript; charset=utf-8",
            ),
            (header::CACHE_CONTROL, "public, max-age=3600"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        AUTOSUBMIT_JS,
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    #[tokio::test]
    async fn serves_the_autosubmit_script_with_cache_and_nosniff_headers() {
        let resp = handle_autosubmit_js(crate::AutosubmitJsPath).await;
        let headers = resp.headers();

        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            "application/javascript; charset=utf-8"
        );
        assert_eq!(
            headers.get(header::CACHE_CONTROL).unwrap(),
            "public, max-age=3600"
        );
        assert_eq!(
            headers.get(header::X_CONTENT_TYPE_OPTIONS).unwrap(),
            "nosniff"
        );

        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body.as_ref(), AUTOSUBMIT_JS.as_bytes());
    }
}
