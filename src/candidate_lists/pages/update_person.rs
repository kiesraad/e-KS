use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppResponse, AppState, Context, CsrfTokens, DbConnection, HtmlTemplate,
    candidate_lists::{
        pages::{CandidateListEditPersonPath, load_candidate_list},
        structs::{CandidateList, CandidateListDetail, CandidateListEntry, MAX_CANDIDATES},
    },
    filters,
    form::{FormData, Validate},
    persons::{repository, structs::PersonForm},
    t,
};

#[derive(Template)]
#[template(path = "candidate_lists/update_person.html")]
struct PersonUpdateTemplate {
    details: CandidateListDetail,
    candidate: CandidateListEntry,
    form: FormData<PersonForm>,
    max_candidates: usize,
}

pub(crate) async fn edit_person_form(
    CandidateListEditPersonPath {
        candidate_list,
        person,
    }: CandidateListEditPersonPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
) -> AppResponse<impl IntoResponse> {
    let details = load_candidate_list(&mut conn, &candidate_list, context.locale).await?;

    let candidate = details
        .candidates
        .iter()
        .find(|c| c.person.id == person)
        .ok_or_else(|| AppError::NotFound("Person not found in candidate list".to_string()))?;

    Ok(HtmlTemplate(
        PersonUpdateTemplate {
            form: FormData::new_with_data(PersonForm::from(candidate.person.clone()), &csrf_tokens),
            candidate: candidate.clone(),
            details,
            max_candidates: MAX_CANDIDATES,
        },
        context,
    ))
}

pub(crate) async fn update_person(
    CandidateListEditPersonPath {
        candidate_list,
        person,
    }: CandidateListEditPersonPath,
    context: Context,
    State(app_state): State<AppState>,
    DbConnection(mut conn): DbConnection,
    form: Form<PersonForm>,
) -> Result<Response, AppError> {
    let details = load_candidate_list(&mut conn, &candidate_list, context.locale).await?;

    let candidate = details
        .candidates
        .iter()
        .find(|c| c.person.id == person)
        .ok_or_else(|| AppError::NotFound("Person not found in candidate list".to_string()))?;

    match form.validate(Some(&candidate.person), app_state.csrf_tokens()) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonUpdateTemplate {
                candidate: candidate.clone(),
                details,
                form: form_data,
                max_candidates: MAX_CANDIDATES,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            repository::update_person(&mut conn, &person).await?;

            // Redirect to the address edit page
            Ok(Redirect::to(&person.edit_address_path()).into_response())
        }
    }
}
