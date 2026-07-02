use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};

use crate::{AppError, AppState};

mod overview;

#[derive(TypedPath)]
#[typed_path("/csb/monitoring", rejection(AppError))]
pub struct CsbMonitoringOverviewPath;

pub fn router() -> Router<AppState> {
    Router::new().typed_get(overview::overview)
}
