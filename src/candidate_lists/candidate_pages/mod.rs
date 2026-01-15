use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;
use uuid::Uuid;

use crate::{AppError, AppState, candidate_lists::Candidate};

mod add;
mod create;
mod delete;
mod edit_address;
mod edit_position;
mod update;

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/candidate-lists/{candidate_list}/reorder/{person}",
    rejection(AppError)
)]
pub struct EditCandidatePositionPath {
    pub candidate_list: Uuid,
    pub person: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{candidate_list}/edit/{person}", rejection(AppError))]
pub struct CandidateListEditPersonPath {
    pub candidate_list: Uuid,
    pub person: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/candidate-lists/{candidate_list}/address/{person}",
    rejection(AppError)
)]
pub struct CandidateListEditAddressPath {
    pub candidate_list: Uuid,
    pub person: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/candidate-lists/{candidate_list}/delete/{person}",
    rejection(AppError)
)]
pub struct CandidateListDeletePersonPath {
    pub candidate_list: Uuid,
    pub person: Uuid,
}

impl Candidate {
    pub fn edit_position_path(&self) -> String {
        EditCandidatePositionPath {
            candidate_list: self.list_id,
            person: self.person.id,
        }
        .to_string()
    }

    pub fn edit_path(&self) -> String {
        CandidateListEditPersonPath {
            candidate_list: self.list_id,
            person: self.person.id,
        }
        .to_string()
    }

    pub fn edit_address_path(&self) -> String {
        CandidateListEditAddressPath {
            candidate_list: self.list_id,
            person: self.person.id,
        }
        .to_string()
    }

    pub fn delete_path(&self) -> String {
        CandidateListDeletePersonPath {
            candidate_list: self.list_id,
            person: self.person.id,
        }
        .to_string()
    }
}

pub fn candidate_router() -> Router<AppState> {
    Router::new()
        .typed_get(add::add_existing_person)
        .typed_post(add::add_person_to_candidate_list)
        .typed_get(edit_position::edit_candidate_position)
        .typed_post(edit_position::update_candidate_position)
        .typed_get(create::new_person_candidate_list)
        .typed_post(create::create_person_candidate_list)
        .typed_get(edit_address::edit_person_address)
        .typed_post(edit_address::update_person_address)
        .typed_get(update::edit_person_form)
        .typed_post(update::update_person)
        .typed_post(delete::delete_person)
}
