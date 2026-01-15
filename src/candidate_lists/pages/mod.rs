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
mod address;
mod create;
mod create_person;
mod delete;
mod edit_position;
mod list;
mod reorder;
mod update;
mod update_person;
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
#[typed_path("/candidate-lists/{id}/edit", rejection(AppError))]
pub(crate) struct CandidateListsEditPath {
    pub(crate) id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{id}/delete", rejection(AppError))]
pub(crate) struct CandidateListsDeletePath {
    pub(crate) id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{id}/add-person", rejection(AppError))]
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

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{candidate_list}/new-person", rejection(AppError))]
pub(crate) struct CandidateListNewPersonPath {
    pub(crate) candidate_list: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{candidate_list}/edit/{person}", rejection(AppError))]
pub(crate) struct CandidateListEditPersonPath {
    pub(crate) candidate_list: Uuid,
    pub(crate) person: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/candidate-lists/{candidate_list}/address/{person}",
    rejection(AppError)
)]
pub(crate) struct CandidateListEditAddressPath {
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

    pub fn update_path(&self) -> String {
        CandidateListsEditPath { id: self.id }.to_uri().to_string()
    }

    pub fn delete_path(&self) -> String {
        CandidateListsDeletePath { id: self.id }
            .to_uri()
            .to_string()
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

    pub fn new_person_path(&self) -> String {
        CandidateListNewPersonPath {
            candidate_list: self.id,
        }
        .to_uri()
        .to_string()
    }

    pub fn edit_person_path(&self, person_id: &Uuid) -> String {
        CandidateListEditPersonPath {
            candidate_list: self.id,
            person: *person_id,
        }
        .to_uri()
        .to_string()
    }

    pub fn edit_person_address_path(&self, person_id: &Uuid) -> String {
        CandidateListEditAddressPath {
            candidate_list: self.id,
            person: *person_id,
        }
        .to_uri()
        .to_string()
    }
}

pub(crate) fn candidate_list_not_found(id: Uuid, locale: Locale) -> AppError {
    AppError::NotFound(t!("candidate_list.not_found", &locale, id))
}

pub fn router() -> Router<AppState> {
    Router::new()
        // manage lists
        .typed_get(list::list_candidate_lists)
        .typed_get(create::new_candidate_list_form)
        .typed_post(create::create_candidate_list)
        // manage single list
        .typed_get(view::view_candidate_list)
        .typed_get(update::edit_candidate_list)
        .typed_post(update::update_candidate_list)
        .typed_post(delete::delete_candidate_list)
        .typed_get(add_person::add_existing_person)
        .typed_post(add_person::add_person_to_candidate_list)
        .typed_post(reorder::reorder_candidate_list)
        // manage person / candidate
        .typed_get(edit_position::edit_candidate_position)
        .typed_post(edit_position::update_candidate_position)
        .typed_get(create_person::new_person_candidate_list)
        .typed_post(create_person::create_person_candidate_list)
        .typed_get(address::edit_person_address_form)
        .typed_post(address::update_person_address)
        .typed_get(update_person::edit_person_form)
        .typed_post(update_person::update_person)
}

pub(super) async fn load_candidate_list(
    conn: &mut PgConnection,
    id: &Uuid,
    locale: Locale,
) -> Result<CandidateListDetail, AppError> {
    repository::get_candidate_list_details(conn, id)
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
    use chrono::{DateTime, NaiveDate, Utc};
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        AppState, Context, CsrfTokens, DbConnection, ElectoralDistrict, Locale, TokenValue,
        candidate_lists::structs::{CandidateListDeleteForm, CandidateListForm},
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
            last_name_prefix: None,
            first_name: Some("Marlon".to_string()),
            initials: "M.B.".to_string(),
            date_of_birth: Some(NaiveDate::from_ymd_opt(1990, 2, 1).unwrap()),
            bsn: None,
            locality: Some("Utrecht".to_string()),
            postal_code: Some("1234 AB".to_string()),
            house_number: Some("10".to_string()),
            house_number_addition: Some("A".to_string()),
            street_name: Some("Stationsstraat".to_string()),
            is_dutch: Some(true),
            custom_country: None,
            custom_region: None,
            address_line_1: None,
            address_line_2: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[sqlx::test]
    async fn new_candidate_list_form_renders_csrf_field(pool: PgPool) -> Result<(), sqlx::Error> {
        let app_state = AppState::new_for_tests(pool.clone());

        let response = create::new_candidate_list_form(
            CandidateListsNewPath {},
            Context::new(Locale::En),
            CsrfTokens::default(),
            DbConnection(pool.acquire().await?),
            State(app_state),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(StatusCode::OK, response.status());
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
            csrf_token: TokenValue("invalid".to_string()),
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

        assert_eq!(StatusCode::OK, response.status());
        let body = response_body_string(response).await;
        assert!(body.contains("Create candidate list"));

        Ok(())
    }

    #[sqlx::test]
    async fn update_candidate_list_persists_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let app_state = AppState::new_for_tests(pool.clone());
        let mut conn = pool.acquire().await.unwrap();
        let csrf_tokens = CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;
        let creation_date = DateTime::from_timestamp(0, 0).unwrap();
        let candidate_list = CandidateList {
            id: Uuid::new_v4(),
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: creation_date,
            updated_at: creation_date,
        };
        repository::create_candidate_list(&mut conn, &candidate_list).await?;

        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::DR],
            csrf_token,
        };
        let response = update::update_candidate_list(
            CandidateListsEditPath {
                id: candidate_list.id,
            },
            Context::new(Locale::En),
            State(app_state),
            csrf_tokens,
            DbConnection(conn),
            Form(form),
        )
        .await
        .unwrap();

        // verify redirect
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");

        // verify updated candidate list object in database
        let mut conn = pool.acquire().await?;
        let lists = repository::list_candidate_list_with_count(&mut conn).await?;
        assert_eq!(lists.len(), 1);

        let updated_list = &lists[0].list;

        assert_eq!(updated_list.view_path(), location);

        assert_eq!(candidate_list.id, updated_list.id);
        assert_eq!(
            vec![ElectoralDistrict::DR],
            updated_list.electoral_districts
        );
        assert_eq!(creation_date, updated_list.created_at);
        // we don't know the exact update date
        // best we can do is to check it at least got updated (i.e. not equal to creation_date)
        assert_ne!(creation_date, updated_list.updated_at);

        Ok(())
    }

    #[sqlx::test]
    async fn update_candidate_list_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let app_state = AppState::new_for_tests(pool.clone());
        let mut conn = pool.acquire().await.unwrap();
        let csrf_tokens = CsrfTokens::default();
        let creation_date = DateTime::from_timestamp(0, 0).unwrap();
        let candidate_list = CandidateList {
            id: Uuid::new_v4(),
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: creation_date,
            updated_at: creation_date,
        };
        repository::create_candidate_list(&mut conn, &candidate_list).await?;

        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::DR],
            csrf_token: TokenValue("invalid".to_string()),
        };
        let response = update::update_candidate_list(
            CandidateListsEditPath {
                id: candidate_list.id,
            },
            Context::new(Locale::En),
            State(app_state),
            csrf_tokens,
            DbConnection(conn),
            Form(form),
        )
        .await
        .unwrap();

        assert_eq!(StatusCode::OK, response.status());
        let body = response_body_string(response).await;
        assert!(body.contains("Edit candidate list"));

        let mut conn = pool.acquire().await?;
        let lists = repository::list_candidate_list_with_count(&mut conn).await?;
        assert_eq!(lists.len(), 1);

        let updated_list = &lists[0].list;

        // verify candidate list didn't update in database
        assert_eq!(&candidate_list, updated_list);

        Ok(())
    }

    #[sqlx::test]
    async fn delete_candidate_list_and_redirect(pool: PgPool) -> Result<(), sqlx::Error> {
        let app_state = AppState::new_for_tests(pool.clone());
        let mut conn = pool.acquire().await.unwrap();
        let csrf_tokens = CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;
        let candidate_list = CandidateList {
            id: Uuid::new_v4(),
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        };
        repository::create_candidate_list(&mut conn, &candidate_list).await?;

        let response = delete::delete_candidate_list(
            CandidateListsDeletePath {
                id: candidate_list.id,
            },
            Context::new(Locale::En),
            State(app_state),
            csrf_tokens,
            DbConnection(conn),
            Form(CandidateListDeleteForm { csrf_token }),
        )
        .await
        .unwrap();

        // verify redirect
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");

        assert_eq!(location, CandidateList::list_path());

        // verify deletion (i.e. no lists in database left)
        let mut conn = pool.acquire().await?;
        let lists = repository::list_candidate_list_with_count(&mut conn).await?;
        assert_eq!(lists.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn delete_candidate_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let app_state = AppState::new_for_tests(pool.clone());
        let mut conn = pool.acquire().await.unwrap();
        let csrf_tokens = CsrfTokens::default();
        let csrf_token = TokenValue("invalid".to_string());
        let candidate_list = CandidateList {
            id: Uuid::new_v4(),
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        };
        repository::create_candidate_list(&mut conn, &candidate_list).await?;

        let response = delete::delete_candidate_list(
            CandidateListsDeletePath {
                id: candidate_list.id,
            },
            Context::new(Locale::En),
            State(app_state),
            csrf_tokens,
            DbConnection(conn),
            Form(CandidateListDeleteForm { csrf_token }),
        )
        .await
        .unwrap();

        // verify redirect
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");

        assert_eq!(location, candidate_list.update_path());

        // verify deletion didn't go through (i.e. still 1 list in database left)
        let mut conn = pool.acquire().await?;
        let lists = repository::list_candidate_list_with_count(&mut conn).await?;
        assert_eq!(lists.len(), 1);

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

        let response = add_person::add_existing_person(
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
        assert_eq!(location, list.edit_person_address_path(&person.id));

        let mut conn = pool.acquire().await?;
        let detail = repository::get_candidate_list_details(&mut conn, &list_id)
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
        repository::update_candidate_list_order(&mut conn, &list_id, &[person_a.id, person_b.id])
            .await?;

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
        let detail = repository::get_candidate_list_details(&mut conn, &list_id)
            .await?
            .expect("candidate list");
        assert_eq!(detail.candidates.len(), 2);
        assert_eq!(detail.candidates[0].person.id, person_b.id);
        assert_eq!(detail.candidates[1].person.id, person_a.id);

        Ok(())
    }
}
