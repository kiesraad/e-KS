use axum::response::{IntoResponse, Redirect, Response};

use crate::{
    AppError, DbConnection,
    persons::{self, structs::Person},
};

use super::DeletePersonPath;

pub(crate) async fn delete_person(
    DeletePersonPath { id }: DeletePersonPath,
    DbConnection(mut conn): DbConnection,
) -> Result<Response, AppError> {
    persons::repository::remove_person(&mut conn, &id).await?;

    Ok(Redirect::to(&Person::list_path()).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{DbConnection, persons, test_utils::sample_person};

    #[sqlx::test]
    async fn delete_person_removes_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4();
        let person = sample_person(id);

        let mut conn = pool.acquire().await?;
        persons::repository::create_person(&mut conn, &person).await?;

        let response = delete_person(DeletePersonPath { id }, DbConnection(pool.acquire().await?))
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
        let found = persons::repository::get_person(&mut conn, &id).await?;
        assert!(found.is_none());

        Ok(())
    }
}
