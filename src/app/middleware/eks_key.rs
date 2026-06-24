//! Optional shared-secret gate. When `Config::eks_key` is set, every request
//! must carry a matching `x-eks-key` header or it is rejected with 401.

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use secrecy::ExposeSecret;
use sha2::{Digest, Sha256};

use crate::AppState;

pub const EKS_KEY_HEADER: &str = "x-eks-key";

pub async fn eks_key_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let Some(expected) = state.config.eks_key.as_ref() else {
        return next.run(request).await;
    };

    let provided = request
        .headers()
        .get(EKS_KEY_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    if keys_match(provided, expected.expose_secret()) {
        next.run(request).await
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

/// Compare two secrets in a bit more constant time. Both inputs are first hashed with
/// SHA-256 so the comparison always runs over fixed-length 32-byte digests,
/// avoiding any length-based early exit.
/// Note that this is not a perfect constant-time comparison, but it should be good enough to prevent trivial timing attacks in this context.
fn keys_match(provided: &str, expected: &str) -> bool {
    let provided_hash = Sha256::digest(provided.as_bytes());
    let expected_hash = Sha256::digest(expected.as_bytes());

    let mut diff = 0u8;
    for (p, e) in provided_hash.iter().zip(expected_hash.iter()) {
        diff |= p ^ e;
    }

    // black_box discourages the optimizer from short-circuiting the check.
    std::hint::black_box(diff) == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, body::Body, http::Request, middleware, routing::get};
    use secrecy::SecretString;
    use tower::ServiceExt;

    use crate::{AppState, Config};

    async fn app_with_key(eks_key: Option<&str>) -> Router {
        let mut config = Config::new_test();
        config.eks_key = eks_key.map(SecretString::from);
        let state = AppState::new_for_tests_with_config(config).await;

        Router::new()
            .route("/probe", get(|| async { "ok" }))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                eks_key_middleware,
            ))
            .with_state(state)
    }

    #[tokio::test]
    async fn passes_through_when_unset() {
        let app = app_with_key(None).await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/probe")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn rejects_missing_header_when_set() {
        let app = app_with_key(Some("s3cret")).await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/probe")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn rejects_wrong_header_when_set() {
        let app = app_with_key(Some("s3cret")).await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/probe")
                    .header(EKS_KEY_HEADER, "nope")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn keys_match_handles_equal_and_unequal_lengths() {
        assert!(keys_match("s3cret", "s3cret"));
        assert!(!keys_match("s3cret", "s3cre"));
        assert!(!keys_match("s3cret", "s3crets"));
        assert!(!keys_match("", "s3cret"));
        assert!(!keys_match("s3cret", ""));
        assert!(keys_match("", ""));
    }

    #[tokio::test]
    async fn accepts_matching_header_when_set() {
        let app = app_with_key(Some("s3cret")).await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/probe")
                    .header(EKS_KEY_HEADER, "s3cret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
