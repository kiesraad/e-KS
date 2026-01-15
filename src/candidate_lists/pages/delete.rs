use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppState, Context, CsrfTokens, DbConnection,
    candidate_lists::{
        pages::{CandidateListsDeletePath, candidate_list_not_found},
        repository,
        structs::{CandidateList, CandidateListDeleteForm},
    },
    form::Validate,
};

pub(crate) async fn delete_candidate_list(
    CandidateListsDeletePath { id }: CandidateListsDeletePath,
    context: Context,
    _: State<AppState>,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
    form: Form<CandidateListDeleteForm>,
) -> Result<Response, AppError> {
    match form.validate(None, &csrf_tokens) {
        Err(_) => {
            // csrf token is invalid => back to edit view
            let candidate_list = repository::get_candidate_list(&mut conn, &id)
                .await?
                .ok_or(candidate_list_not_found(id, context.locale))?;
            Ok(Redirect::to(&candidate_list.update_path()).into_response())
        }
        Ok(_) => {
            repository::remove_candidate_list(&mut conn, id).await?;
            Ok(Redirect::to(&CandidateList::list_path()).into_response())
        }
    }
}
