use askama::Template;
use axum::{extract::State, response::IntoResponse};

use super::{CandidateList, CandidateListsPath};

use crate::{
    AppError, AppState, Context, DbConnection, ElectionConfig, HtmlTemplate, Locale,
    candidate_lists::{self, structs::CandidateListSummary},
    filters,
    persons::{self, structs::Person},
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

pub(crate) async fn list_candidate_lists(
    _: CandidateListsPath,
    context: Context,
    State(app_state): State<AppState>,
    DbConnection(mut conn): DbConnection,
) -> Result<impl IntoResponse, AppError> {
    let candidate_lists =
        candidate_lists::repository::list_candidate_list_with_count(&mut conn).await?;
    let total_persons = persons::repository::count_persons(&mut conn).await?;
    let election = app_state.config().election;

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
    use axum::{extract::State, http::StatusCode, response::IntoResponse};
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        AppState, Context, DbConnection, Locale, candidate_lists,
        test_utils::{response_body_string, sample_candidate_list},
    };

    #[sqlx::test]
    async fn list_candidate_lists_shows_created_list(pool: PgPool) -> Result<(), sqlx::Error> {
        let list = sample_candidate_list(Uuid::new_v4());
        let mut conn = pool.acquire().await?;
        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;

        let response = list_candidate_lists(
            CandidateListsPath {},
            Context::new(Locale::En),
            State(AppState::new_for_tests(pool.clone())),
            DbConnection(pool.acquire().await?),
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
