use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{AppError, AppState, Locale, t};

use super::{
    repository,
    structs::{CandidateList, CandidateListDetail},
};

mod add_person;
mod create;
mod edit_position;
mod list;
mod reorder;
mod view;

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists", rejection(AppError))]
pub(crate) struct CandidateListsPath;

#[derive(TypedPath)]
#[typed_path("/candidate-lists/new", rejection(AppError))]
pub(crate) struct CandidateListsNewPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{id}", rejection(AppError))]
pub(crate) struct ViewCandidateListPath {
    pub(crate) id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{id}/add", rejection(AppError))]
pub(crate) struct CandidateListAddPersonPath {
    pub(crate) id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{id}/reorder", rejection(AppError))]
pub(crate) struct CandidateListReorderPath {
    pub(crate) id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/candidate-lists/{candidate_list}/reorder/{person}",
    rejection(AppError)
)]
pub(crate) struct EditCandidatePositionPath {
    pub(crate) candidate_list: Uuid,
    pub(crate) person: Uuid,
}

impl CandidateList {
    pub fn list_path() -> String {
        CandidateListsPath {}.to_uri().to_string()
    }

    pub fn new_path() -> String {
        CandidateListsNewPath {}.to_uri().to_string()
    }

    pub fn add_person_path(&self) -> String {
        CandidateListAddPersonPath { id: self.id }
            .to_uri()
            .to_string()
    }

    pub fn edit_candidate_position_path(&self, person_id: &Uuid) -> String {
        EditCandidatePositionPath {
            candidate_list: self.id,
            person: *person_id,
        }
        .to_uri()
        .to_string()
    }

    pub fn view_path(&self) -> String {
        ViewCandidateListPath { id: self.id }.to_uri().to_string()
    }

    pub fn reorder_path(&self) -> String {
        CandidateListReorderPath { id: self.id }
            .to_uri()
            .to_string()
    }
}

fn candidate_list_not_found(id: Uuid, locale: Locale) -> AppError {
    AppError::NotFound(t!("candidate_list.not_found", &locale, id))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(list::list_candidate_lists)
        .typed_get(create::new_candidate_list_form)
        .typed_post(create::create_candidate_list)
        .typed_get(view::view_candidate_list)
        .typed_get(add_person::add_existing_person_to_candidate_list)
        .typed_post(add_person::add_person_to_candidate_list)
        .typed_get(edit_position::edit_candidate_position)
        .typed_post(edit_position::update_candidate_position)
        .typed_post(reorder::reorder_candidate_list)
}

pub(super) async fn load_candidate_list(
    conn: &mut PgConnection,
    id: &Uuid,
    locale: Locale,
) -> Result<CandidateListDetail, AppError> {
    repository::get_candidate_list(conn, id)
        .await?
        .ok_or_else(|| candidate_list_not_found(*id, locale))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Json,
        extract::State,
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::extract::Form;
    use chrono::{NaiveDate, Utc};
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        AppState, Context, CsrfTokens, DbConnection, ElectoralDistrict, Locale,
        candidate_lists::structs::CandidateListForm,
        persons::{
            repository as persons_repository,
            structs::{Gender, Person},
        },
        test_utils::response_body_string,
    };

    fn sample_candidate_list(id: Uuid) -> CandidateList {
        CandidateList {
            id,
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_person(id: Uuid, last_name: &str) -> Person {
        Person {
            id,
            gender: Some(Gender::Female),
            last_name: last_name.to_string(),
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

    #[sqlx::test]
    async fn new_candidate_list_form_renders_csrf_field(pool: PgPool) -> Result<(), sqlx::Error> {
        let app_state = AppState::new_for_tests(pool);

        let response = create::new_candidate_list_form(
            CandidateListsNewPath {},
            Context::new(Locale::En),
            CsrfTokens::default(),
            State(app_state),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("name=\"csrf_token\""));
        assert!(body.contains("action=\"/candidate-lists/new\""));

        Ok(())
    }

    #[sqlx::test]
    async fn create_candidate_list_persists_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let app_state = AppState::new_for_tests(pool.clone());
        let csrf_tokens = CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;
        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::UT],
            csrf_token,
        };

        let response = create::create_candidate_list(
            CandidateListsNewPath {},
            Context::new(Locale::En),
            State(app_state),
            csrf_tokens,
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

        let mut conn = pool.acquire().await?;
        let lists = repository::list_candidate_list_with_count(&mut conn).await?;
        assert_eq!(lists.len(), 1);
        assert_eq!(location, lists[0].list.view_path());

        Ok(())
    }

    #[sqlx::test]
    async fn create_candidate_list_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let app_state = AppState::new_for_tests(pool.clone());
        let csrf_tokens = CsrfTokens::default();
        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::UT],
            csrf_token: "invalid".to_string(),
        };

        let response = create::create_candidate_list(
            CandidateListsNewPath {},
            Context::new(Locale::En),
            State(app_state),
            csrf_tokens,
            DbConnection(pool.acquire().await?),
            Form(form),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Create candidate list"));

        Ok(())
    }

    #[sqlx::test]
    async fn list_candidate_lists_shows_created_list(pool: PgPool) -> Result<(), sqlx::Error> {
        let list = sample_candidate_list(Uuid::new_v4());
        let mut conn = pool.acquire().await?;
        repository::create_candidate_list(&mut conn, &list).await?;

        let response = list::list_candidate_lists(
            CandidateListsPath {},
            Context::new(Locale::En),
            State(AppState::new_for_tests(pool.clone())),
            DbConnection(pool.acquire().await?),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Utrecht"));

        Ok(())
    }

    #[sqlx::test]
    async fn view_candidate_list_renders_persons(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = Uuid::new_v4();
        let list = sample_candidate_list(list_id);
        let person = sample_person(Uuid::new_v4(), "Doe");

        let mut conn = pool.acquire().await?;
        repository::create_candidate_list(&mut conn, &list).await?;
        persons_repository::create_person(&mut conn, &person).await?;

        let response = add_person::add_existing_person_to_candidate_list(
            CandidateListAddPersonPath { id: list_id },
            Context::new(Locale::En),
            DbConnection(pool.acquire().await?),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&list.add_person_path()));
        assert!(body.contains("Doe"));

        Ok(())
    }

    #[sqlx::test]
    async fn add_person_to_candidate_list_adds_and_redirects(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let list_id = Uuid::new_v4();
        let list = sample_candidate_list(list_id);
        let person = sample_person(Uuid::new_v4(), "Roe");

        let mut conn = pool.acquire().await?;
        repository::create_candidate_list(&mut conn, &list).await?;
        persons_repository::create_person(&mut conn, &person).await?;

        let response = add_person::add_person_to_candidate_list(
            CandidateListAddPersonPath { id: list_id },
            Context::new(Locale::En),
            DbConnection(pool.acquire().await?),
            Form(add_person::AddPersonForm {
                person_id: person.id,
            }),
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
        assert_eq!(location, list.view_path());

        let mut conn = pool.acquire().await?;
        let detail = repository::get_candidate_list(&mut conn, &list_id)
            .await?
            .expect("candidate list");
        assert_eq!(detail.candidates.len(), 1);
        assert_eq!(detail.candidates[0].person.id, person.id);

        Ok(())
    }

    #[sqlx::test]
    async fn reorder_candidate_list_updates_positions(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = Uuid::new_v4();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person(Uuid::new_v4(), "Alpha");
        let person_b = sample_person(Uuid::new_v4(), "Beta");

        let mut conn = pool.acquire().await?;
        repository::create_candidate_list(&mut conn, &list).await?;
        persons_repository::create_person(&mut conn, &person_a).await?;
        persons_repository::create_person(&mut conn, &person_b).await?;
        repository::update_candidate_list(&mut conn, &list_id, &[person_a.id, person_b.id]).await?;

        let response = reorder::reorder_candidate_list(
            CandidateListReorderPath { id: list_id },
            Context::new(Locale::En),
            DbConnection(pool.acquire().await?),
            Json(reorder::CandidateListReorderPayload {
                person_ids: vec![person_b.id, person_a.id],
            }),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let mut conn = pool.acquire().await?;
        let detail = repository::get_candidate_list(&mut conn, &list_id)
            .await?
            .expect("candidate list");
        assert_eq!(detail.candidates.len(), 2);
        assert_eq!(detail.candidates[0].person.id, person_b.id);
        assert_eq!(detail.candidates[1].person.id, person_a.id);

        Ok(())
    }
}
