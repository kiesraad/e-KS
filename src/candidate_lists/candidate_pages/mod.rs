use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;
use uuid::Uuid;

use crate::{AppError, AppState, candidate_lists::CandidateList};

mod add_candidate;
mod create_candidate;
mod delete_candidate;
mod edit_candidate_address;
mod edit_candidate_position;
mod update_candidate;

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
        .typed_get(add_candidate::add_existing_person)
        .typed_post(add_candidate::add_person_to_candidate_list)
        .typed_get(edit_candidate_position::edit_candidate_position)
        .typed_post(edit_candidate_position::update_candidate_position)
        .typed_get(create_candidate::new_person_candidate_list)
        .typed_post(create_candidate::create_person_candidate_list)
        .typed_get(edit_candidate_address::edit_person_address)
        .typed_post(edit_candidate_address::update_person_address)
        .typed_get(update_candidate::edit_person_form)
        .typed_post(update_candidate::update_person)
        .typed_post(delete_candidate::delete_person)
}
