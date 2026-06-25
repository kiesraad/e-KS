use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{AppError, AppState, StreamId, csb::examination::extractors::CsbPoliticalGroup};

mod overview;
mod political_group;

#[derive(TypedPath)]
#[typed_path("/csb/examination", rejection(AppError))]
pub struct CsbExaminationOverviewPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/csb/examination/{stream_id}", rejection(AppError))]
pub struct CsbPoliticalGroupPath {
    pub stream_id: StreamId,
}

impl CsbPoliticalGroup {
    pub fn examination_path(&self) -> impl TypedPath {
        CsbPoliticalGroupPath {
            stream_id: self.stream_id,
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(overview::overview)
        .typed_get(political_group::overview)
}
