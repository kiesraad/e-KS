use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{
    AppError, AppState, QueryParamState, StreamId, csb::examination::extractors::CsbPoliticalGroup,
};

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

#[derive(TypedPath, Deserialize)]
#[typed_path("/csb/examination/{stream_id}/toggle-finish", rejection(AppError))]
pub struct CsbPoliticalGroupToggleFinishPath {
    pub stream_id: StreamId,
}

impl CsbPoliticalGroup {
    pub fn examination_path(&self) -> impl TypedPath {
        CsbPoliticalGroupPath {
            stream_id: self.stream_id,
        }
    }

    pub fn examination_toggle_finish_path(&self) -> impl TypedPath {
        CsbPoliticalGroupToggleFinishPath {
            stream_id: self.stream_id,
        }
    }

    pub fn after_toggle_finish_examination_path(&self) -> impl TypedPath {
        CsbPoliticalGroupPath {
            stream_id: self.stream_id,
        }
        .with_query_params(QueryParamState::success())
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(overview::overview)
        .typed_get(political_group::overview)
        .typed_post(political_group::toggle_examination_finish)
}
