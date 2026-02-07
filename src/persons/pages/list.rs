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
        AppError, AppStore, Context,
        pagination::Pagination,
        persons::PersonId,
        test_utils::{response_body_string, sample_person},
    };

    #[sqlx::test]
    async fn list_persons_shows_created_person(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let id = PersonId::new();
        let person = sample_person(id);

        person.create(&store).await?;

        let response = list_persons(
            PersonsPath {},
            Context::new_test_without_db(),
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
