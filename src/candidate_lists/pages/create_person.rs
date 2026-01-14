use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use uuid::Uuid;

use crate::{
    AppError, AppState, Context, CsrfTokens, DbConnection, HtmlTemplate,
    candidate_lists::{
        pages::{CandidateListNewPersonPath, load_candidate_list},
        repository,
        structs::{CandidateList, CandidateListDetail, MAX_CANDIDATES},
    },
    filters,
    form::{FormData, Validate},
    persons::{self, structs::PersonForm},
    t,
};

#[derive(Template)]
#[template(path = "candidate_lists/create_person.html")]
struct PersonCreateTemplate {
    details: CandidateListDetail,
    form: FormData<PersonForm>,
    max_candidates: usize,
}

pub(crate) async fn new_person_candidate_list(
    CandidateListNewPersonPath { candidate_list }: CandidateListNewPersonPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
) -> Result<impl IntoResponse, AppError> {
    let details: CandidateListDetail =
        load_candidate_list(&mut conn, &candidate_list, context.locale).await?;

    Ok(HtmlTemplate(
        PersonCreateTemplate {
            details,
            form: FormData::new(&csrf_tokens),
            max_candidates: MAX_CANDIDATES,
        },
        context,
    )
    .into_response())
}

pub(crate) async fn create_person_candidate_list(
    CandidateListNewPersonPath { candidate_list }: CandidateListNewPersonPath,
    context: Context,
    State(app_state): State<AppState>,
    DbConnection(mut conn): DbConnection,
    form: Form<PersonForm>,
) -> Result<Response, AppError> {
    let details: CandidateListDetail =
        load_candidate_list(&mut conn, &candidate_list, context.locale).await?;

    match form.validate(None, app_state.csrf_tokens()) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonCreateTemplate {
                details,
                form: form_data,
                max_candidates: MAX_CANDIDATES,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            let person = persons::repository::create_person(&mut conn, &person).await?;

            let mut person_ids: Vec<Uuid> =
                details.candidates.iter().map(|c| c.person.id).collect();
            person_ids.push(person.id);
            repository::update_candidate_list(&mut conn, &candidate_list, &person_ids).await?;

            Ok(Redirect::to(&details.list.edit_person_address_path(&person.id)).into_response())
        }
    }
}
