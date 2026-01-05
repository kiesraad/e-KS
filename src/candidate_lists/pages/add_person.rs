use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::Form;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppError, Context, DbConnection, candidate_lists::repository,
    persons::repository as persons_repository, t,
};

use super::{CandidateListAddPersonPath, load_candidate_list};

#[derive(Deserialize)]
pub(crate) struct AddPersonForm {
    pub person_id: Uuid,
}

pub(crate) async fn add_person_to_candidate_list(
    CandidateListAddPersonPath { id }: CandidateListAddPersonPath,
    context: Context,
    DbConnection(mut conn): DbConnection,
    Form(form): Form<AddPersonForm>,
) -> Result<Response, AppError> {
    let detail = load_candidate_list(&mut conn, &id, context.locale).await?;

    if detail
        .candidates
        .iter()
        .any(|c| c.person.id == form.person_id)
    {
        return Ok(Redirect::to(&detail.list.view_path()).into_response());
    }

    let person = persons_repository::get_person(&mut conn, &form.person_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(t!("person.not_found", &context.locale, form.person_id))
        })?;

    let mut person_ids: Vec<Uuid> = detail.candidates.iter().map(|c| c.person.id).collect();
    person_ids.push(person.id);

    let updated = repository::update_candidate_list(&mut conn, &id, &person_ids).await?;

    Ok(Redirect::to(&updated.list.view_path()).into_response())
}
