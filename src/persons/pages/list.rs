use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, Context, HtmlTemplate, filters,
    persons::{Person, PersonPagination, PersonSort, pages::PersonsPath},
};

#[derive(Template)]
#[template(path = "persons/list.html")]
struct PersonListTemplate {
    person_pagination: PersonPagination,
}

pub async fn list_persons(
    _: PersonsPath,
    context: Context,
    person_pagination: PersonPagination,
) -> Result<impl IntoResponse, AppError> {
    Ok(HtmlTemplate(
        PersonListTemplate { person_pagination },
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
        pagination::Pagination,
        persons::{self, PersonId},
        test_utils::{response_body_string, sample_person},
    };

    #[sqlx::test]
    async fn list_persons_shows_created_person(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = PersonId::new();
        let person = sample_person(id);

        persons::create_person(&pool, &person).await?;

        let response = list_persons(
            PersonsPath {},
            Context::new_test(pool.clone()).await,
            PersonPagination {
                persons: vec![person],
                pagination: Pagination::default().set_total(1),
            },
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));

        Ok(())
    }
}
