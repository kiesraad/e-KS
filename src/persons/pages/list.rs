use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, Context, DbConnection, HtmlTemplate, filters,
    pagination::{Pagination, PaginationInfo},
    persons::{self, Person, PersonSort, pages::PersonsPath},
    t,
};

#[derive(Template)]
#[template(path = "persons/list.html")]
struct PersonListTemplate {
    persons: Vec<Person>,
    pagination: PaginationInfo<PersonSort>,
}

pub async fn list_persons(
    _: PersonsPath,
    context: Context,
    pagination: Pagination<PersonSort>,
    DbConnection(mut conn): DbConnection,
) -> Result<impl IntoResponse, AppError> {
    let total_items = persons::repository::count_persons(&mut conn).await?.max(0) as u64;
    let pagination = pagination.set_total(total_items);

    let persons = persons::repository::list_persons(
        &mut conn,
        pagination.limit(),
        pagination.offset(),
        pagination.sort(),
        pagination.direction(),
    )
    .await?;

    Ok(HtmlTemplate(
        PersonListTemplate {
            persons,
            pagination,
        },
        context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{http::StatusCode, response::IntoResponse};
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        Context, DbConnection, Locale,
        pagination::Pagination,
        persons,
        test_utils::{response_body_string, sample_person},
    };

    #[sqlx::test]
    async fn list_persons_shows_created_person(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4();
        let person = sample_person(id);

        let mut conn = pool.acquire().await?;
        persons::repository::create_person(&mut conn, &person).await?;

        let response = list_persons(
            PersonsPath {},
            Context::new(Locale::En),
            Pagination::default(),
            DbConnection(pool.acquire().await?),
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
