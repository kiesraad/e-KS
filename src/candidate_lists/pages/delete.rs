use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppState, Context, CsrfTokens, DbConnection,
    candidate_lists::{
        pages::{
            CandidateListsDeletePath, CandidateListsEditPath, update::edit_candidate_list_form,
        },
        repository,
        structs::{CandidateList, CandidateListDeleteForm},
    },
    form::Validate,
};

pub(crate) async fn delete_candidate_list(
    CandidateListsDeletePath { id }: CandidateListsDeletePath,
    context: Context,
    state: State<AppState>,
    csrf_tokens: CsrfTokens,
    mut db_conn: DbConnection,
    form: Form<CandidateListDeleteForm>,
) -> Result<Response, AppError> {
    match form.validate(None, &csrf_tokens) {
        Err(_) => {
            // csrf token is invalid => back to edit view
            edit_candidate_list_form(
                CandidateListsEditPath { id },
                context,
                csrf_tokens,
                db_conn,
                state,
            )
            .await
        }
        Ok(_) => {
            repository::remove_candidate_list(&mut db_conn.0, id).await?;
            Ok(Redirect::to(&CandidateList::list_path()).into_response())
        }
    }
}
