//! Health check endpoint reporting whether the app can reach its persistence
//! backend (typically the configured database).

use axum::{Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};

use crate::AppState;

pub fn health_router() -> Router<AppState> {
    Router::new().route("/health", get(health_handler))
}

async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    match state.store_registry.persistence().health_check().await {
        Ok(()) => (StatusCode::OK, "healthy"),
        Err(err) => {
            tracing::warn!("health check failed: {err}");
            (StatusCode::SERVICE_UNAVAILABLE, "unhealthy")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use crate::{AppState, test_utils::response_body_string};

    #[tokio::test]
    async fn health_returns_ok_for_reachable_backend() {
        let state = AppState::new_for_tests().await;
        let app: Router = health_router().with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert_eq!(body, "healthy");
    }
}
