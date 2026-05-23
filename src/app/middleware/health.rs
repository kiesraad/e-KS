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
