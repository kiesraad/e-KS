use axum::response::{IntoResponse, Redirect, Response};

use crate::{
    AppError, DbConnection,
    persons::{self, Person, pages::DeletePersonPath},
};

pub async fn delete_person(
    DeletePersonPath { person_id }: DeletePersonPath,
    DbConnection(mut conn): DbConnection,
) -> Result<Response, AppError> {
    persons::remove_person(&mut conn, person_id).await?;

    Ok(Redirect::to(&Person::list_path()).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    use crate::{
        DbConnection,
        persons::{self, PersonId},
        test_utils::sample_person,
    };

    #[sqlx::test]
    async fn delete_person_removes_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        let mut conn = pool.acquire().await?;
        persons::create_person(&mut conn, &person).await?;

        let response = delete_person(
            DeletePersonPath { person_id },
            DbConnection(pool.acquire().await?),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(axum::http::header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");
        assert_eq!(location, Person::list_path());

        let mut conn = pool.acquire().await?;
        let found = persons::get_person(&mut conn, person_id).await?;
        assert!(found.is_none());

        Ok(())
    }
}
