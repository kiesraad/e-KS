use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppState, Context, CsrfTokens, DbConnection, HtmlTemplate,
    candidate_lists::{
        self,
        pages::{CandidateListNewPersonPath, load_candidate_list},
        structs::{CandidateList, FullCandidateList, MAX_CANDIDATES},
    },
    filters,
    form::{FormData, Validate},
    persons::{self, structs::PersonForm},
    t,
};

#[derive(Template)]
#[template(path = "candidate_lists/create_person.html")]
struct PersonCreateTemplate {
    full_list: FullCandidateList,
    form: FormData<PersonForm>,
    max_candidates: usize,
}

pub(crate) async fn new_person_candidate_list(
    CandidateListNewPersonPath { candidate_list }: CandidateListNewPersonPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
) -> Result<impl IntoResponse, AppError> {
    let full_list: FullCandidateList =
        load_candidate_list(&mut conn, &candidate_list, context.locale).await?;

    Ok(HtmlTemplate(
        PersonCreateTemplate {
            full_list,
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
    let full_list: FullCandidateList =
        load_candidate_list(&mut conn, &candidate_list, context.locale).await?;

    match form.validate(None, app_state.csrf_tokens()) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonCreateTemplate {
                full_list,
                form: form_data,
                max_candidates: MAX_CANDIDATES,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            let person = persons::repository::create_person(&mut conn, &person).await?;

            let mut person_ids = full_list.get_ids();
            person_ids.push(person.id);
            candidate_lists::repository::update_candidate_list_order(
                &mut conn,
                &candidate_list,
                &person_ids,
            )
            .await?;

            Ok(Redirect::to(&full_list.list.edit_person_address_path(&person.id)).into_response())
        }
    }
}
