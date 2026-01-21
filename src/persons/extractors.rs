use axum::extract::{FromRef, FromRequestParts, Path};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    AppError, Context,
    pagination::Pagination,
    persons::{self, Person, PersonId, PersonPagination, PersonSort},
    t,
};

#[derive(Deserialize)]
struct PersonPathParams {
    #[serde(alias = "person_id")]
    person_id: PersonId,
}

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
        let Path(PersonPathParams { person_id }) =
            Path::<PersonPathParams>::from_request_parts(parts, state).await?;

        let person = persons::get_person(&mut conn, person_id)
            .await?
            .ok_or(AppError::NotFound(t!(
                "person.not_found",
                context.locale,
                person_id
            )))?;

        Ok(person)
    }
}

impl<S> FromRequestParts<S> for PersonPagination
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
        let pagination: Pagination<PersonSort> =
            Pagination::from_request_parts(parts, state).await?;

        let total_items = persons::count_persons(&mut conn).await?.max(0) as u64;
        let pagination = pagination.set_total(total_items);

        let persons = persons::list_persons(
            &mut conn,
            pagination.limit(),
            pagination.offset(),
            pagination.sort(),
            pagination.direction(),
        )
        .await?;

        Ok(PersonPagination {
            persons,
            pagination,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode, header},
        middleware,
        routing::get,
    };
    use sqlx::PgPool;
    use tower::ServiceExt;

    use crate::{
        AppState, Locale, persons, render_error_pages, t,
        test_utils::{response_body_string, sample_person},
    };

    #[sqlx::test]
    async fn person_extractor_loads_person(pool: PgPool) {
        let person = sample_person(PersonId::new());
        let mut conn = pool.acquire().await.unwrap();
        persons::create_person(&mut conn, &person).await.unwrap();

        let app = Router::new()
            .route(
                "/persons/{person_id}",
                get(|person: Person| async { person.last_name }),
            )
            .with_state(AppState::new_for_tests(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/persons/{}", person.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));
    }

    #[sqlx::test]
    async fn person_extractor_returns_not_found(pool: PgPool) {
        let person_id = PersonId::new();

        let app = Router::new()
            .route(
                "/persons/{person_id}",
                get(|person: Person| async { person.last_name }),
            )
            .layer(middleware::from_fn(render_error_pages))
            .with_state(AppState::new_for_tests(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/persons/{}", person_id))
                    .header(header::ACCEPT_LANGUAGE, "en")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response_body_string(response).await;

        let expected = t!("person.not_found", Locale::En, person_id);
        assert!(body.contains(&expected));
    }
}
