use askama::Template;
use axum::{extract::State, response::IntoResponse};

use crate::{
    AppError, AppStore, Context, HtmlTemplate,
    candidate_lists::{CandidateList, CandidateListSummary, pages::CandidateListsPath},
    filters,
    persons::Person,
};

#[derive(Template)]
#[template(path = "candidate_lists/list.html")]
struct CandidateListIndexTemplate {
    candidate_lists: Vec<CandidateListSummary>,
    total_persons: usize,
}

pub async fn list_candidate_lists(
    _: CandidateListsPath,
    context: Context,
    State(store): State<AppStore>,
) -> Result<impl IntoResponse, AppError> {
    let candidate_lists = CandidateListSummary::list(&store)?;
    let total_persons = store.get_person_count()?;

    Ok(HtmlTemplate(
        CandidateListIndexTemplate {
            candidate_lists,
            total_persons,
        },
        context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{http::StatusCode, response::IntoResponse};
    use sqlx::PgPool;

    use crate::{
        AppStore, Context,
        candidate_lists::CandidateListId,
        test_utils::{response_body_string, sample_candidate_list},
    };

    #[sqlx::test]
    async fn list_candidate_lists_shows_created_list(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        let response = list_candidate_lists(
            CandidateListsPath {},
            Context::new_test_without_db(),
            State(store),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Utrecht"));

        Ok(())
    }
}
