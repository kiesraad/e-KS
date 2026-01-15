use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    AppError, AppState, Locale,
    candidate_lists::{self},
    t,
};

use super::structs::{CandidateList, FullCandidateList};

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
        .typed_get(address::edit_person_address)
        .typed_post(address::update_person_address)
        .typed_get(update_person::edit_person_form)
        .typed_post(update_person::update_person)
}

pub(crate) fn candidate_list_not_found(id: Uuid, locale: Locale) -> AppError {
    AppError::NotFound(t!("candidate_list.not_found", &locale, id))
}

pub(super) async fn load_candidate_list(
    conn: &mut PgConnection,
    id: &Uuid,
    locale: Locale,
) -> Result<FullCandidateList, AppError> {
    candidate_lists::repository::get_full_candidate_list(conn, id)
        .await?
        .ok_or_else(|| candidate_list_not_found(*id, locale))
}
