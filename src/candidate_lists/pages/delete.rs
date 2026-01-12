use axum::response::{IntoResponse, Redirect, Response};

use crate::{
    AppError, DbConnection,
    candidate_lists::{pages::CandidateListsDeletePath, repository, structs::CandidateList},
};

pub(crate) async fn delete_candidate_list(
    CandidateListsDeletePath { id }: CandidateListsDeletePath,
    DbConnection(mut conn): DbConnection,
) -> Result<Response, AppError> {
    repository::remove_candidate_list(&mut conn, id).await?;

    Ok(Redirect::to(&CandidateList::list_path()).into_response())
}
