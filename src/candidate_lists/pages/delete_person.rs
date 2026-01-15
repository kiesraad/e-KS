use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppState, Context, CsrfTokens, DbConnection,
    candidate_lists::{
        self,
        pages::{CandidateListDeletePersonPath, CandidateListEditPersonPath},
        structs::CandidateList,
    },
    form::{EmptyForm, Validate},
    persons,
};

pub(crate) async fn delete_person(
    CandidateListDeletePersonPath {
        candidate_list,
        person,
    }: CandidateListDeletePersonPath,
    context: Context,
    _: State<AppState>,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
    form: Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate(None, &csrf_tokens) {
        Err(_) => {
            // csrf token is invalid => back to edit view
            Ok(Redirect::to(
                &CandidateListEditPersonPath {
                    candidate_list,
                    person,
                }
                .to_string(),
            )
            .into_response())
        }
        Ok(_) => {
            let full_list = candidate_lists::pages::load_candidate_list(
                &mut conn,
                &candidate_list,
                context.locale,
            )
            .await?;
            let candidate = full_list.get_candidate(&person, context.locale)?;

            // remove person from list
            let mut updates_ids = full_list.get_ids();
            updates_ids.retain(|id| id != &candidate.person.id);
            candidate_lists::repository::update_candidate_list_order(
                &mut conn,
                &candidate_list,
                &updates_ids,
            )
            .await?;

            persons::repository::remove_person(&mut conn, &candidate.person.id).await?;

            Ok(Redirect::to(&CandidateList::list_path()).into_response())
        }
    }
}
