use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use uuid::Uuid;

use crate::{
    AppError, AppState, Context, CsrfTokens, DbConnection, ElectoralDistrict, HtmlTemplate, Locale,
    candidate_lists::{
        pages::CandidateListsEditPath,
        repository,
        structs::{CandidateList, CandidateListForm},
    },
    filters,
    form::{FormData, Validate},
    t,
};

#[derive(Template)]
#[template(path = "candidate_lists/update.html")]
struct CandidateListUpdateTemplate {
    form: FormData<CandidateListForm>,
    candidate_list: CandidateList,
    electoral_districts: &'static [ElectoralDistrict],
}

fn candidate_list_not_found(id: Uuid, locale: Locale) -> AppError {
    AppError::NotFound(t!("candidate_list.not_found", &locale, id))
}

pub(crate) async fn edit_candidate_list_form(
    CandidateListsEditPath { id }: CandidateListsEditPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let electoral_districts = app_state.config().get_districts();

    let candidate_list = repository::get_candidate_list(&mut conn, &id)
        .await?
        .ok_or(candidate_list_not_found(id, context.locale))?;

    Ok(HtmlTemplate(
        CandidateListUpdateTemplate {
            form: FormData::new_with_data(
                CandidateListForm::from(candidate_list.clone()),
                &csrf_tokens,
            ),
            candidate_list,
            electoral_districts,
        },
        context,
    )
    .into_response())
}

pub(crate) async fn update_candidate_list(
    CandidateListsEditPath { id }: CandidateListsEditPath,
    context: Context,
    State(app_state): State<AppState>,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
    Form(form): Form<CandidateListForm>,
) -> Result<Response, AppError> {
    let electoral_districts = app_state.config().election.electoral_districts();

    let candidate_list = repository::get_candidate_list(&mut conn, &id)
        .await?
        .ok_or(candidate_list_not_found(id, context.locale))?;

    match form.validate(None, &csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            CandidateListUpdateTemplate {
                form: form_data,
                candidate_list,
                electoral_districts,
            },
            context,
        )
        .into_response()),
        Ok(candidate_list) => {
            let candidate_list =
                repository::update_candidate_list(&mut conn, &candidate_list).await?;
            Ok(Redirect::to(&candidate_list.view_path()).into_response())
        }
    }
}
