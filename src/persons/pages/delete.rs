use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use sqlx::PgPool;

use crate::{
    AppError,
    persons::{self, Person, pages::DeletePersonPath},
};

pub async fn delete_person(
    DeletePersonPath { person_id }: DeletePersonPath,
    State(pool): State<PgPool>,
) -> Result<Response, AppError> {
    persons::remove_person(&pool, person_id).await?;

    Ok(Redirect::to(&Person::list_path()).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    use crate::{
        persons::{self, PersonId},
        test_utils::sample_person,
    };

    #[sqlx::test]
    async fn delete_person_removes_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        persons::create_person(&pool, &person).await?;

        let response = delete_person(DeletePersonPath { person_id }, State(pool.clone()))
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

        let found = persons::get_person(&pool, person_id).await?;
        assert!(found.is_none());

        Ok(())
    }
}
