use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppError, AppState,
    pagination::Pagination,
    persons::structs::{Person, PersonSort},
};

mod create;
mod delete;
mod list;
mod update;

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons", rejection(AppError))]
pub(crate) struct PersonsPath;

#[derive(TypedPath)]
#[typed_path("/persons/new", rejection(AppError))]
pub(crate) struct PersonsNewPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons/{id}", rejection(AppError))]
pub(crate) struct EditPersonPath {
    pub(crate) id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons/{id}/delete", rejection(AppError))]
pub(crate) struct DeletePersonPath {
    pub(crate) id: Uuid,
}

impl Person {
    pub fn list_path() -> String {
        PersonsPath {}.to_uri().to_string()
    }

    pub fn list_path_with_pagination(pagination: &Pagination<PersonSort>) -> String {
        format!("{}{}", PersonsPath {}.to_uri(), pagination.as_query())
    }

    pub fn new_path() -> String {
        PersonsNewPath {}.to_uri().to_string()
    }

    pub fn edit_path(&self) -> String {
        EditPersonPath { id: self.id }.to_uri().to_string()
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(list::list_persons)
        .typed_post(create::create_person)
        .typed_get(create::new_person_form)
        .typed_get(update::edit_person_form)
        .typed_post(update::update_person)
        .typed_post(delete::delete_person)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        extract::State,
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::extract::Form;
    use chrono::{NaiveDate, Utc};
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        AppState, Context, CsrfTokens, DbConnection, Locale,
        pagination::{Pagination, SortDirection},
        persons::{
            repository,
            structs::{Gender, Person, PersonForm, PersonSort},
        },
        test_utils::response_body_string,
    };

    fn sample_person(id: Uuid) -> Person {
        Person {
            id,
            gender: Some(Gender::Female),
            last_name: "Doe".to_string(),
            first_name: Some("Marlon".to_string()),
            initials: "M.B.".to_string(),
            date_of_birth: Some(NaiveDate::from_ymd_opt(1990, 2, 1).unwrap()),
            locality: Some("Utrecht".to_string()),
            postal_code: Some("1234 AB".to_string()),
            house_number: Some("10".to_string()),
            house_number_addition: Some("A".to_string()),
            street_name: Some("Stationsstraat".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_form(csrf_token: &str) -> PersonForm {
        PersonForm {
            gender: "female".to_string(),
            last_name: "Doe".to_string(),
            first_name: "Marlon".to_string(),
            initials: "M.B.".to_string(),
            date_of_birth: "01-02-1990".to_string(),
            locality: "Utrecht".to_string(),
            postal_code: "1234 AB".to_string(),
            house_number: "10".to_string(),
            house_number_addition: "A".to_string(),
            street_name: "Stationsstraat".to_string(),
            csrf_token: csrf_token.to_string(),
        }
    }

    #[tokio::test]
    async fn new_person_form_renders_csrf_field() {
        let context = Context::new(Locale::En);
        let csrf_tokens = CsrfTokens::default();

        let response = create::new_person_form(PersonsNewPath {}, context, csrf_tokens)
            .await
            .unwrap()
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_string(response).await;
        assert!(body.contains("name=\"csrf_token\""));
        assert!(body.contains("action=\"/persons/new\""));
    }

    #[sqlx::test]
    async fn create_person_persists_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let app_state = AppState::new_for_tests(pool.clone());
        let context = Context::new(Locale::En);
        let csrf_token = app_state.csrf_tokens().issue().value;
        let form = sample_form(&csrf_token);

        let response = create::create_person(
            PersonsNewPath {},
            context,
            State(app_state),
            DbConnection(pool.acquire().await?),
            Form(form),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");
        let pagination = Pagination {
            sort: PersonSort::CreatedAt,
            order: SortDirection::Desc,
            ..Default::default()
        };
        assert_eq!(location, Person::list_path_with_pagination(&pagination));

        let mut conn = pool.acquire().await?;
        let count = repository::count_persons(&mut conn).await?;
        assert_eq!(count, 1);

        Ok(())
    }

    #[sqlx::test]
    async fn create_person_invalid_form_renders_template(pool: PgPool) -> Result<(), sqlx::Error> {
        let app_state = AppState::new_for_tests(pool.clone());
        let context = Context::new(Locale::En);
        let csrf_token = app_state.csrf_tokens().issue().value;
        let mut form = sample_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = create::create_person(
            PersonsNewPath {},
            context,
            State(app_state),
            DbConnection(pool.acquire().await?),
            Form(form),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("This field must not be empty."));

        Ok(())
    }

    #[sqlx::test]
    async fn list_persons_shows_created_person(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4();
        let person = sample_person(id);

        let mut conn = pool.acquire().await?;
        repository::create_person(&mut conn, &person).await?;

        let response = list::list_persons(
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
        assert!(body.contains("Doe"));

        Ok(())
    }

    #[sqlx::test]
    async fn edit_person_form_renders_existing_person(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4();
        let person = sample_person(id);

        let mut conn = pool.acquire().await?;
        repository::create_person(&mut conn, &person).await?;

        let response = update::edit_person_form(
            EditPersonPath { id },
            Context::new(Locale::En),
            CsrfTokens::default(),
            DbConnection(pool.acquire().await?),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Doe"));

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_persists_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4();
        let person = sample_person(id);

        let mut conn = pool.acquire().await?;
        repository::create_person(&mut conn, &person).await?;

        let app_state = AppState::new_for_tests(pool.clone());
        let csrf_token = app_state.csrf_tokens().issue().value;
        let mut form = sample_form(&csrf_token);
        form.last_name = "Updated".to_string();

        let response = update::update_person(
            EditPersonPath { id },
            Context::new(Locale::En),
            State(app_state),
            DbConnection(pool.acquire().await?),
            Form(form),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");
        let pagination = Pagination {
            sort: PersonSort::UpdatedAt,
            order: SortDirection::Desc,
            ..Default::default()
        };
        assert_eq!(location, Person::list_path_with_pagination(&pagination));

        let mut conn = pool.acquire().await?;
        let updated = repository::get_person(&mut conn, &id)
            .await?
            .expect("updated person");
        assert_eq!(updated.last_name, "Updated");

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_invalid_form_renders_template(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4();
        let person = sample_person(id);

        let mut conn = pool.acquire().await?;
        repository::create_person(&mut conn, &person).await?;

        let app_state = AppState::new_for_tests(pool.clone());
        let csrf_token = app_state.csrf_tokens().issue().value;
        let mut form = sample_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = update::update_person(
            EditPersonPath { id },
            Context::new(Locale::En),
            State(app_state),
            DbConnection(pool.acquire().await?),
            Form(form),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("This field must not be empty."));

        Ok(())
    }

    #[sqlx::test]
    async fn delete_person_removes_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4();
        let person = sample_person(id);

        let mut conn = pool.acquire().await?;
        repository::create_person(&mut conn, &person).await?;

        let response =
            delete::delete_person(DeletePersonPath { id }, DbConnection(pool.acquire().await?))
                .await
                .unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");
        assert_eq!(location, Person::list_path());

        let mut conn = pool.acquire().await?;
        let found = repository::get_person(&mut conn, &id).await?;
        assert!(found.is_none());

        Ok(())
    }
}
