use askama::Template;
use axum::{extract::State, response::IntoResponse};
use sqlx::PgPool;

use crate::{
    AppError, Context, HtmlTemplate,
    candidate_lists::{self, CandidateList, CandidateListSummary, pages::CandidateListsPath},
    filters, persons,
    persons::Person,
};

#[derive(Template)]
#[template(path = "candidate_lists/list.html")]
struct CandidateListIndexTemplate {
    candidate_lists: Vec<CandidateListSummary>,
    total_persons: i64,
}

pub async fn list_candidate_lists(
    _: CandidateListsPath,
    context: Context,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, AppError> {
    let candidate_lists = candidate_lists::list_candidate_list_with_count(&pool).await?;
    let total_persons = persons::count_persons(&pool).await?;

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
        Context,
        candidate_lists::{self, CandidateListId},
        test_utils::{response_body_string, sample_candidate_list},
    };

    #[sqlx::test]
    async fn list_candidate_lists_shows_created_list(pool: PgPool) -> Result<(), sqlx::Error> {
        let list = sample_candidate_list(CandidateListId::new());
        candidate_lists::create_candidate_list(&pool, &list).await?;

        let response = list_candidate_lists(
            CandidateListsPath {},
            Context::new_test(pool.clone()).await,
            State(pool.clone()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Utrecht"));

        Ok(())
    }
}
