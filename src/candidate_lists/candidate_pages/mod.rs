use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;
use uuid::Uuid;

use crate::{AppError, AppState, candidate_lists::structs::CandidateList};

mod add_person;
mod address;
mod create_person;
mod delete_person;
mod edit_position;
mod update_person;

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

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/candidate-lists/{candidate_list}/delete/{person}",
    rejection(AppError)
)]
pub(crate) struct CandidateListDeletePersonPath {
    pub(crate) candidate_list: Uuid,
    pub(crate) person: Uuid,
}

impl CandidateList {
    pub fn edit_candidate_position_path(&self, person_id: &Uuid) -> String {
        EditCandidatePositionPath {
            candidate_list: self.id,
            person: *person_id,
        }
        .to_string()
    }

    pub fn new_person_path(&self) -> String {
        CandidateListNewPersonPath {
            candidate_list: self.id,
        }
        .to_string()
    }

    pub fn edit_person_path(&self, person_id: &Uuid) -> String {
        CandidateListEditPersonPath {
            candidate_list: self.id,
            person: *person_id,
        }
        .to_string()
    }

    pub fn edit_person_address_path(&self, person_id: &Uuid) -> String {
        CandidateListEditAddressPath {
            candidate_list: self.id,
            person: *person_id,
        }
        .to_string()
    }

    pub fn delete_person_path(&self, person_id: &Uuid) -> String {
        CandidateListDeletePersonPath {
            candidate_list: self.id,
            person: *person_id,
        }
        .to_string()
    }
}

pub fn candidate_router() -> Router<AppState> {
    Router::new()
        .typed_get(add_person::add_existing_person)
        .typed_post(add_person::add_person_to_candidate_list)
        .typed_get(edit_position::edit_candidate_position)
        .typed_post(edit_position::update_candidate_position)
        .typed_get(create_person::new_person_candidate_list)
        .typed_post(create_person::create_person_candidate_list)
        .typed_get(address::edit_person_address)
        .typed_post(address::update_person_address)
        .typed_get(update_person::edit_person_form)
        .typed_post(update_person::update_person)
        .typed_post(delete_person::delete_person)
}
