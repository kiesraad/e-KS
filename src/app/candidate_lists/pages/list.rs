use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, Context, HtmlTemplate, RequestCtx,
    candidate_lists::{CandidateList, CandidateListSummary, pages::CandidateListsPath},
    filters,
    persons::Person,
};

#[derive(Template)]
#[template(path = "candidate_lists/pages/list.html")]
struct CandidateListIndexTemplate {
    candidate_lists: Vec<CandidateListSummary>,
    total_persons: usize,
}

pub async fn list_candidate_lists(
    _: CandidateListsPath,
    ctx: RequestCtx,
) -> Result<impl IntoResponse, AppError> {
    let candidate_lists = CandidateListSummary::list(&ctx.store)?;
    let total_persons = ctx.store.get_person_count();

    Ok(HtmlTemplate(
        CandidateListIndexTemplate {
            candidate_lists,
            total_persons,
        },
        ctx.context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppStore, Context, QueryParamState,
        candidate_lists::CandidateListId,
        test_utils::{response_body_string, sample_candidate_list},
    };
    use axum::{http::StatusCode, response::IntoResponse};

    #[tokio::test]
    async fn list_candidate_lists_shows_created_list() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        let response = list_candidate_lists(
            CandidateListsPath {},
            RequestCtx {
                context: Context::new_test_without_db(),
                store,
                query: QueryParamState::default(),
            },
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Utrecht"));

        Ok(())
    }
}
