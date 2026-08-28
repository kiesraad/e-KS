//! Public http-01 challenge endpoint, dialled by the CA's validation servers.

use axum::{
    Router,
    extract::{FromRef, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{AppError, acme::AcmeStore};

#[derive(TypedPath, Deserialize)]
#[typed_path("/.well-known/acme-challenge/{token}", rejection(AppError))]
pub struct AcmeChallengePath {
    pub token: String,
}

/// Generic over the router state, requiring only that an [`AcmeStore`] can be
/// borrowed from it.
pub fn acme_challenge_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AcmeStore: FromRef<S>,
{
    Router::new().typed_get(acme_challenge)
}

/// Public by protocol; unknown or expired tokens 404.
async fn acme_challenge(
    AcmeChallengePath { token }: AcmeChallengePath,
    State(store): State<AcmeStore>,
) -> Response {
    match store.find_challenge(&token).await {
        Some(key_authorization) => key_authorization.into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_utils::response_body_string;

    #[tokio::test]
    async fn serves_stored_key_authorization() {
        let store = AcmeStore::default();
        store.put_challenge("tok", "tok.thumbprint").await.unwrap();

        let response = acme_challenge(
            AcmeChallengePath {
                token: "tok".to_string(),
            },
            State(store),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_body_string(response).await, "tok.thumbprint");
    }

    #[tokio::test]
    async fn unknown_token_is_not_found() {
        let response = acme_challenge(
            AcmeChallengePath {
                token: "unknown".to_string(),
            },
            State(AcmeStore::default()),
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
