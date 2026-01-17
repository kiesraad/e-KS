use axum::extract::{FromRef, FromRequestParts, Path};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    AppError, Context,
    candidate_lists::{self, Candidate, CandidateList, CandidateListId, FullCandidateList},
    persons::PersonId,
    t,
};

#[derive(Deserialize)]
struct CandidateListPathParams {
    #[serde(alias = "list_id")]
    list_id: CandidateListId,
}

#[derive(Deserialize)]
struct CandidateListAndPersonPathParams {
    #[serde(alias = "list_id")]
    list_id: CandidateListId,
    #[serde(alias = "person_id")]
    person_id: PersonId,
}

impl<S> FromRequestParts<S> for CandidateList
where
    S: Send + Sync,
    PgPool: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let mut conn = PgPool::from_ref(state).acquire().await?;
        let context = Context::from_request_parts(parts, state)
            .await
            .unwrap_or_default();
        let Path(CandidateListPathParams { list_id }) =
            Path::<CandidateListPathParams>::from_request_parts(parts, state).await?;

        let candidate_list = candidate_lists::repository::get_candidate_list(&mut conn, list_id)
            .await?
            .ok_or(AppError::NotFound(t!(
                "candidate_list.not_found",
                context.locale,
                list_id
            )))?;

        Ok(candidate_list)
    }
}

impl<S> FromRequestParts<S> for FullCandidateList
where
    S: Send + Sync,
    PgPool: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let mut conn = PgPool::from_ref(state).acquire().await?;
        let context = Context::from_request_parts(parts, state)
            .await
            .unwrap_or_default();
        let Path(CandidateListPathParams { list_id }) =
            Path::<CandidateListPathParams>::from_request_parts(parts, state).await?;

        let full_list = candidate_lists::repository::get_full_candidate_list(&mut conn, list_id)
            .await?
            .ok_or(AppError::NotFound(t!(
                "candidate_list.not_found",
                context.locale,
                list_id
            )))?;

        Ok(full_list)
    }
}

impl<S> FromRequestParts<S> for Candidate
where
    S: Send + Sync,
    PgPool: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let mut conn = PgPool::from_ref(state).acquire().await?;
        let context = Context::from_request_parts(parts, state)
            .await
            .unwrap_or_default();
        let Path(CandidateListAndPersonPathParams { list_id, person_id }) =
            Path::<CandidateListAndPersonPathParams>::from_request_parts(parts, state).await?;

        let candidate = candidate_lists::repository::get_candidate(&mut conn, list_id, person_id)
            .await
            .map_err(|err| match err {
                sqlx::Error::RowNotFound => AppError::NotFound(
                    t!("person.not_found_in_candidate_list", context.locale).to_string(),
                ),
                _ => err.into(),
            })?;

        Ok(candidate)
    }
}
