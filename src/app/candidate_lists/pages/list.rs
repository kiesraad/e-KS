use askama::Template;
use axum::{extract::State, response::IntoResponse};

use crate::{
    AppError, Context, HtmlTemplate, Store,
    candidate_lists::{CandidateList, CandidateListSummary, pages::CandidateListsPath},
    filters,
    persons::Person,
};

#[derive(Template)]
#[template(path = "candidate_lists//pages/list.html")]
struct CandidateListIndexTemplate {
    candidate_lists: Vec<CandidateListSummary>,
    total_persons: usize,
}

pub async fn list_candidate_lists(
    _: CandidateListsPath,
    context: Context,
    State(store): State<Store>,
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
    use crate::{
        Context, Store,
        candidate_lists::CandidateListId,
        test_utils::{response_body_string, sample_candidate_list},
    };
    use axum::{http::StatusCode, response::IntoResponse};

    #[tokio::test]
    async fn list_candidate_lists_shows_created_list() -> Result<(), AppError> {
        let store = Store::new_for_test().await;
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
