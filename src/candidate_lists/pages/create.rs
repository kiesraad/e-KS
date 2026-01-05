use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppState, Context, CsrfTokens, DbConnection, ElectoralDistrict, HtmlTemplate,
    candidate_lists::structs::CandidateListForm,
    filters,
    form::{FormData, Validate},
    t,
};

use super::{CandidateList, CandidateListsNewPath, repository};

#[derive(Template)]
#[template(path = "candidate_lists/create.html")]
struct CandidateListCreateTemplate {
    form: FormData<CandidateListForm>,
    electoral_districts: &'static [ElectoralDistrict],
}

pub(crate) async fn new_candidate_list_form(
    _: CandidateListsNewPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let electoral_districts = app_state.config().get_districts();

    Ok(HtmlTemplate(
        CandidateListCreateTemplate {
            form: FormData::new(&csrf_tokens),
            electoral_districts,
        },
        context,
    )
    .into_response())
}

pub(crate) async fn create_candidate_list(
    _: CandidateListsNewPath,
    context: Context,
    State(app_state): State<AppState>,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
    Form(form): Form<CandidateListForm>,
) -> Result<Response, AppError> {
    let electoral_districts = app_state.config().election.electoral_districts();

    match form.validate(None, &csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            CandidateListCreateTemplate {
                form: form_data,
                electoral_districts,
            },
            context,
        )
        .into_response()),
        Ok(candidate_list) => {
            let candidate_list =
                repository::create_candidate_list(&mut conn, &candidate_list).await?;
            Ok(Redirect::to(&candidate_list.view_path()).into_response())
        }
    }
}
