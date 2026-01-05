use askama::Template;
use axum::{extract::State, response::IntoResponse};

use super::{CandidateListsPath, repository};

use crate::{
    AppError, AppState, Context, DbConnection, ElectionConfig, HtmlTemplate, Locale,
    candidate_lists::structs::CandidateListSummary, filters, t,
};

#[derive(Template)]
#[template(path = "candidate_lists/list.html")]
struct CandidateListIndexTemplate {
    candidate_lists: Vec<CandidateListSummary>,
    election: ElectionConfig,
    locale: Locale,
}

pub(crate) async fn list_candidate_lists(
    _: CandidateListsPath,
    context: Context,
    State(app_state): State<AppState>,
    DbConnection(mut conn): DbConnection,
) -> Result<impl IntoResponse, AppError> {
    let candidate_lists = repository::list_candidate_list_with_count(&mut conn).await?;
    let election = app_state.config().election;

    Ok(HtmlTemplate(
        CandidateListIndexTemplate {
            candidate_lists,
            election,
            locale: context.locale,
        },
        context,
    ))
}
