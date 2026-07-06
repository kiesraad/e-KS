//! CSRF token verification for every mutating request. Runs inside the session
//! middleware, so no handler can forget the check.

use axum::{
    body::{Body, Bytes, to_bytes},
    extract::{FromRequest, Multipart, Request},
    http::{Method, StatusCode, header::CONTENT_TYPE, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{AppError, Session};

/// Form field carrying the CSRF token.
pub const CSRF_FORM_FIELD: &str = "csrf_token";

/// Body-buffering cap: above the 5 MiB import limit; handler-level
/// `DefaultBodyLimit`s still apply downstream.
const MAX_BUFFERED_BODY_BYTES: usize = 6 * 1024 * 1024;

/// Rejects mutating requests whose form body lacks the session's CSRF token.
pub async fn csrf_middleware(request: Request, next: Next) -> Response {
    if matches!(
        request.method(),
        &Method::GET | &Method::HEAD | &Method::OPTIONS
    ) {
        return next.run(request).await;
    }

    // The session middleware always runs before this one on these routes.
    let Some(session) = request.extensions().get::<Session>().cloned() else {
        return AppError::InternalServerError.into_response();
    };

    let (parts, body) = request.into_parts();
    let Ok(bytes) = to_bytes(body, MAX_BUFFERED_BODY_BYTES).await else {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    };

    match submitted_token(&parts, &bytes).await {
        Some(token) if session.csrf_matches(&token) => {
            let request = Request::from_parts(parts, Body::from(bytes));
            next.run(request).await
        }
        _ => AppError::CsrfTokenInvalid.into_response(),
    }
}

/// Extract the submitted token from a buffered form body; `None` rejects.
async fn submitted_token(parts: &Parts, bytes: &Bytes) -> Option<String> {
    let content_type = parts.headers.get(CONTENT_TYPE)?.to_str().ok()?;

    if content_type.starts_with("application/x-www-form-urlencoded") {
        return url::form_urlencoded::parse(bytes)
            .find(|(name, _)| name == CSRF_FORM_FIELD)
            .map(|(_, value)| value.into_owned());
    }

    if content_type.starts_with("multipart/form-data") {
        return multipart_token(parts, bytes.clone()).await;
    }

    None
}

/// Find the `csrf_token` field in an already-buffered multipart body.
async fn multipart_token(parts: &Parts, bytes: Bytes) -> Option<String> {
    let request = Request::from_parts(parts.clone(), Body::from(bytes));
    let mut multipart = Multipart::from_request(request, &()).await.ok()?;

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some(CSRF_FORM_FIELD) {
            return field.text().await.ok();
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, middleware, routing::post};
    use tower::ServiceExt;

    async fn ok_handler() -> &'static str {
        "ok"
    }

    fn app_with_session() -> (Router, Session) {
        let session = Session::new_test();
        let app = Router::new()
            .route("/submit", post(ok_handler))
            .layer(middleware::from_fn(csrf_middleware))
            .layer(middleware::from_fn({
                let session = session.clone();
                move |mut request: Request, next: Next| {
                    let session = session.clone();
                    async move {
                        request.extensions_mut().insert(session);
                        next.run(request).await
                    }
                }
            }));
        (app, session)
    }

    fn urlencoded_post(body: String) -> Request {
        Request::builder()
            .method("POST")
            .uri("/submit")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(body))
            .unwrap()
    }

    /// A POST carrying the session's token in an urlencoded body passes and the
    /// body reaches the handler intact.
    #[tokio::test]
    async fn accepts_valid_urlencoded_token() {
        let (app, session) = app_with_session();

        let request = urlencoded_post(format!("a=b&csrf_token={}", session.csrf_token()));
        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    /// A wrong token is rejected before the handler runs.
    #[tokio::test]
    async fn rejects_invalid_token() {
        let (app, _session) = app_with_session();

        let request = urlencoded_post("csrf_token=wrong".to_string());
        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// A POST without any token is rejected.
    #[tokio::test]
    async fn rejects_missing_token() {
        let (app, _session) = app_with_session();

        let request = urlencoded_post("a=b".to_string());
        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// The token is also found in multipart bodies (file uploads).
    #[tokio::test]
    async fn accepts_valid_multipart_token() {
        let (app, session) = app_with_session();

        let boundary = "test-boundary";
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"csrf_token\"\r\n\r\n{}\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file_data\"; filename=\"a.csv\"\r\nContent-Type: text/csv\r\n\r\ndata\r\n--{boundary}--\r\n",
            session.csrf_token()
        );
        let request = Request::builder()
            .method("POST")
            .uri("/submit")
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap();
        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    /// Safe methods pass through without a token.
    #[tokio::test]
    async fn get_requests_pass_without_token() {
        let session = Session::new_test();
        let app = Router::new()
            .route("/page", axum::routing::get(ok_handler))
            .layer(middleware::from_fn(csrf_middleware))
            .layer(middleware::from_fn(
                move |mut request: Request, next: Next| {
                    let session = session.clone();
                    async move {
                        request.extensions_mut().insert(session);
                        next.run(request).await
                    }
                },
            ));

        let request = Request::builder().uri("/page").body(Body::empty()).unwrap();
        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }
}
