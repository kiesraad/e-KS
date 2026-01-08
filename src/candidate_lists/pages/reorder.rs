use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::{AppError, Context, DbConnection, candidate_lists::repository};

use super::{CandidateListReorderPath, load_candidate_list};

#[derive(Deserialize)]
pub(crate) struct CandidateListReorderPayload {
    pub person_ids: Vec<Uuid>,
}

pub(crate) async fn reorder_candidate_list(
    CandidateListReorderPath { id }: CandidateListReorderPath,
    context: Context,
    DbConnection(mut conn): DbConnection,
    Json(payload): Json<CandidateListReorderPayload>,
) -> Result<impl IntoResponse, AppError> {
    load_candidate_list(&mut conn, &id, context.locale).await?;
    repository::update_candidate_list_order(&mut conn, &id, &payload.person_ids).await?;

    Ok(StatusCode::NO_CONTENT)
}
