use axum::extract::{FromRef, FromRequestParts};
use sqlx::PgPool;

use crate::{
    AppError,
    pagination::Pagination,
    persons::{self, PersonPagination, PersonSort},
};

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
        http::{Request, StatusCode},
        routing::get,
    };
    use sqlx::PgPool;
    use tower::ServiceExt;

    use crate::{
        AppState,
        persons::{self, PersonId},
        test_utils::{response_body_string, sample_person_with_last_name},
    };

    #[sqlx::test]
    async fn person_pagination_extractor_slices_and_orders(pool: PgPool) {
        let mut conn = pool.acquire().await.unwrap();
        persons::create_person(
            &mut conn,
            &sample_person_with_last_name(PersonId::new(), "Bakker"),
        )
        .await
        .unwrap();
        persons::create_person(
            &mut conn,
            &sample_person_with_last_name(PersonId::new(), "Jansen"),
        )
        .await
        .unwrap();

        let app = Router::new()
            .route(
                "/persons",
                get(|pagination: PersonPagination| async move {
                    let last_name = pagination
                        .persons
                        .first()
                        .map(|person| person.last_name.clone())
                        .unwrap_or_default();
                    format!("{}:{}", pagination.pagination.page, last_name)
                }),
            )
            .with_state(AppState::new_for_tests(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/persons?page=2&per_page=1&sort=last_name&order=asc")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("2:Jansen"));
    }
}
