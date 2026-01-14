use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppState, Context, CsrfTokens, DbConnection, HtmlTemplate,
    candidate_lists::{
        pages::{CandidateListNewPersonPath, load_candidate_list},
        structs::{CandidateList, CandidateListDetail, MAX_CANDIDATES},
    },
    filters,
    form::{FormData, Validate},
    persons::{repository, structs::PersonForm},
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
    match form.validate(None, app_state.csrf_tokens()) {
        Err(form_data) => {
            let details: CandidateListDetail =
                load_candidate_list(&mut conn, &candidate_list, context.locale).await?;

            Ok(HtmlTemplate(
                PersonCreateTemplate {
                    details,
                    form: form_data,
                    max_candidates: MAX_CANDIDATES,
                },
                context,
            )
            .into_response())
        }
        Ok(person) => {
            repository::create_person(&mut conn, &person).await?;

            Ok(Redirect::to(&person.edit_address_path()).into_response())
        }
    }
}
