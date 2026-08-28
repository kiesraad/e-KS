use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};

use crate::{AppError, AppRequestState};

mod overview;

#[derive(TypedPath)]
#[typed_path("/csb/monitoring", rejection(AppError))]
pub struct CsbMonitoringOverviewPath;

pub fn router<S: AppRequestState>() -> Router<S> {
    Router::new().typed_get(overview::overview)
}
