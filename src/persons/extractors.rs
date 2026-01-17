use axum::extract::{FromRef, FromRequestParts, Path};
use sqlx::PgPool;

use crate::{
    AppError, Context,
    persons::{self, Person, PersonId},
    t,
};

impl<S> FromRequestParts<S> for Person
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
        let Path(person_id) = Path::<PersonId>::from_request_parts(parts, state).await?;

        let person = persons::repository::get_person(&mut conn, person_id)
            .await?
            .ok_or(AppError::NotFound(t!(
                "person.not_found",
                context.locale,
                person_id
            )))?;

        Ok(person)
    }
}
