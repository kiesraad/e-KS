//! Health check endpoints: [`health_router`] reports whether the app can reach
//! its persistence backend (typically the configured database),
//! [`lb_health_router`] only that this process is up and serving HTTP.

use axum::{Router, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::routing::{RouterExt, TypedPath};

use crate::{AppError, AppState};

#[derive(TypedPath)]
#[typed_path("/health", rejection(AppError))]
pub struct HealthPath;

#[derive(TypedPath)]
#[typed_path("/lb-health", rejection(AppError))]
pub struct LbHealthPath;

pub fn health_router() -> Router<AppState> {
    Router::new().typed_get(health_handler)
}

pub fn lb_health_router() -> Router<AppState> {
    Router::new().typed_get(lb_health_handler)
}

async fn health_handler(_: HealthPath, State(state): State<AppState>) -> impl IntoResponse {
    match state.store_registry.persistence().health_check().await {
        Ok(()) => (StatusCode::OK, "healthy"),
        Err(err) => {
            tracing::warn!("health check failed: {err}");
            (StatusCode::SERVICE_UNAVAILABLE, "unhealthy")
        }
    }
}

async fn lb_health_handler(_: LbHealthPath) -> impl IntoResponse {
    (StatusCode::OK, "started")
}
