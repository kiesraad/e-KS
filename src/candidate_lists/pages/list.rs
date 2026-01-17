use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, Config, Context, DbConnection, ElectionConfig, HtmlTemplate, Locale,
    candidate_lists::{self, CandidateList, CandidateListSummary, pages::CandidateListsPath},
    filters,
    persons::{self, Person},
    t,
};

#[derive(Template)]
#[template(path = "candidate_lists/list.html")]
struct CandidateListIndexTemplate {
    candidate_lists: Vec<CandidateListSummary>,
    election: ElectionConfig,
    total_persons: i64,
    locale: Locale,
}

pub async fn list_candidate_lists(
    _: CandidateListsPath,
    context: Context,
    DbConnection(mut conn): DbConnection,
    config: Config,
) -> Result<impl IntoResponse, AppError> {
    let candidate_lists =
        candidate_lists::repository::list_candidate_list_with_count(&mut conn).await?;
    let total_persons = persons::repository::count_persons(&mut conn).await?;
    let election = config.election;

    Ok(HtmlTemplate(
        CandidateListIndexTemplate {
            candidate_lists,
            election,
            locale: context.locale,
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
        Context, DbConnection, Locale,
        candidate_lists::{self, CandidateListId},
        test_utils::{response_body_string, sample_candidate_list},
    };

    #[sqlx::test]
    async fn list_candidate_lists_shows_created_list(pool: PgPool) -> Result<(), sqlx::Error> {
        let config = Config::new_test();
        let list = sample_candidate_list(CandidateListId::new());
        let mut conn = pool.acquire().await?;
        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;

        let response = list_candidate_lists(
            CandidateListsPath {},
            Context::new(Locale::En),
            DbConnection(pool.acquire().await?),
            config,
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
