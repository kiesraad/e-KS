//! Typed paths for the "Herstelde lijsten" (recovery) routes. They mirror the
//! examination routes under their own prefix; the pages themselves are the
//! examination pages rendered in [`CsbPhase::Recovery`](crate::structs::csb::CsbPhase)
//! mode.

use axum_extra::routing::TypedPath;
use serde::Deserialize;

use crate::{
    AppError, StreamId,
    structs::{candidate_lists::CandidateListId, csb::OmissionId, persons::PersonId},
};

#[derive(TypedPath)]
#[typed_path("/csb/recovery", rejection(AppError))]
pub struct CsbRecoveryOverviewPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/csb/recovery/{stream_id}", rejection(AppError))]
pub struct CsbRecoveryPoliticalGroupPath {
    pub stream_id: StreamId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/csb/recovery/{stream_id}/general-information", rejection(AppError))]
pub struct CsbRecoveryGeneralInformationPath {
    pub stream_id: StreamId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/csb/recovery/{stream_id}/list/{list_id}", rejection(AppError))]
pub struct CsbRecoveryCandidateListPath {
    pub stream_id: StreamId,
    pub list_id: CandidateListId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/csb/recovery/{stream_id}/list/{list_id}/candidate/{person_id}",
    rejection(AppError)
)]
pub struct CsbRecoveryCandidatePath {
    pub stream_id: StreamId,
    pub list_id: CandidateListId,
    pub person_id: PersonId,
}

/// The recovery todo page: every omission of the political group with its
/// recovered / not-recovered control.
#[derive(TypedPath, Deserialize)]
#[typed_path("/csb/recovery/{stream_id}/omissions", rejection(AppError))]
pub struct CsbRecoveryOmissionsPath {
    pub stream_id: StreamId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/csb/recovery/{stream_id}/omission/{omission_id}/status",
    rejection(AppError)
)]
pub struct CsbSetOmissionStatusPath {
    pub stream_id: StreamId,
    pub omission_id: OmissionId,
}
