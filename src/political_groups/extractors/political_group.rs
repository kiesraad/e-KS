use axum::extract::{FromRef, FromRequestParts};
use sqlx::PgPool;

use crate::{
    AppError,
    political_groups::{self, PoliticalGroup},
};

impl<S> FromRequestParts<S> for PoliticalGroup
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);

        political_groups::get_single_political_group(&pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Political group not found.".to_string()))
    }
}
