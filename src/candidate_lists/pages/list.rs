use askama::Template;
use axum::{extract::State, response::IntoResponse};

use super::{CandidateList, CandidateListsPath, repository};

use crate::{
    AppError, AppState, Context, DbConnection, ElectionConfig, HtmlTemplate, Locale,
    candidate_lists::structs::CandidateListSummary,
    filters,
    persons::{repository as person_repository, structs::Person},
    t,
};

#[derive(Template)]
#[template(path = "candidate_lists/list.html")]
struct CandidateListIndexTemplate {
    candidate_lists: Vec<CandidateListSummary>,
    election: ElectionConfig,
    total_persons: String,
    locale: Locale,
}

pub(crate) async fn list_candidate_lists(
    _: CandidateListsPath,
    context: Context,
    State(app_state): State<AppState>,
    DbConnection(mut conn): DbConnection,
) -> Result<impl IntoResponse, AppError> {
    let candidate_lists = repository::list_candidate_list_with_count(&mut conn).await?;
    let total_persons = person_repository::count_persons(&mut conn).await?;
    let election = app_state.config().election;

    Ok(HtmlTemplate(
        CandidateListIndexTemplate {
            candidate_lists,
            election,
            locale: context.locale,
            total_persons: total_persons.to_string(),
        },
        context,
    ))
}
