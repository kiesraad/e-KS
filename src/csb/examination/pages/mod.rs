use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppError, AppState, QueryParamState, StreamId,
    candidate_lists::CandidateListId,
    csb::{OmissionType, examination::extractors::CsbPoliticalGroup},
};

mod general_information;
mod omission;
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

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/csb/examination/{stream_id}/general-information",
    rejection(AppError)
)]
pub struct CsbGeneralInformationPath {
    pub stream_id: StreamId,
}

/// The "add omission" dialog. Renders (GET) and handles (POST) the overlay form
/// that adds an omission to the item identified by `omission_type` + `reference`
/// within the political group's examination stream (`stream_id`).
#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/csb/examination/{stream_id}/omission/{omission_type}/{reference}",
    rejection(AppError)
)]
pub struct CsbAddOmissionPath {
    pub stream_id: StreamId,
    pub omission_type: OmissionType,
    pub reference: Uuid,
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

    pub fn general_information_path(&self) -> impl TypedPath {
        CsbGeneralInformationPath {
            stream_id: self.stream_id,
        }
    }

    /// Path to the dialog that adds a general (political group level) omission.
    pub fn add_political_group_omission_path(&self) -> impl TypedPath {
        CsbAddOmissionPath {
            stream_id: self.stream_id,
            omission_type: OmissionType::PoliticalGroup,
            reference: self.stream_id.into(),
        }
    }

    /// Path to the dialog that adds an omission to a specific candidate list.
    pub fn add_candidate_list_omission_path(&self, list: &CandidateListId) -> impl TypedPath {
        CsbAddOmissionPath {
            stream_id: self.stream_id,
            omission_type: OmissionType::CandidateList,
            reference: (*list).into(),
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
        .typed_get(general_information::overview)
        .typed_get(omission::add_omission)
        .typed_post(omission::add_omission_submit)
}
