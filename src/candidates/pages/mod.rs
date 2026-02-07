use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{
    AppError, AppState,
    candidate_lists::CandidateListId,
    candidates::Candidate,
    persons::{InitialEditQuery, PersonId},
};

mod add;
mod create;
mod delete;
mod update;
mod update_address;
mod update_position;
mod update_representative;

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{list_id}/reorder/{person_id}", rejection(AppError))]
pub struct UpdateCandidatePositionPath {
    pub list_id: CandidateListId,
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/candidate-lists/{list_id}/representative/{person_id}",
    rejection(AppError)
)]
pub struct UpdateRepresentativePath {
    pub list_id: CandidateListId,
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{list_id}/update/{person_id}", rejection(AppError))]
pub struct CandidateListUpdatePersonPath {
    pub list_id: CandidateListId,
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{list_id}/address/{person_id}", rejection(AppError))]
pub struct CandidateListUpdateAddressPath {
    pub list_id: CandidateListId,
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{list_id}/delete/{person_id}", rejection(AppError))]
pub struct CandidateListDeletePersonPath {
    pub list_id: CandidateListId,
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{list_id}/add", rejection(AppError))]
pub struct AddCandidatePath {
    pub list_id: CandidateListId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{list_id}/create", rejection(AppError))]
pub struct CreateCandidatePath {
    pub list_id: CandidateListId,
}

impl Candidate {
    pub fn update_position_path(&self) -> String {
        UpdateCandidatePositionPath {
            list_id: self.list_id,
            person_id: self.person.id,
        }
        .to_string()
    }

    pub fn update_path(&self) -> String {
        CandidateListUpdatePersonPath {
            list_id: self.list_id,
            person_id: self.person.id,
        }
        .to_string()
    }

    pub fn update_address_path(&self) -> String {
        CandidateListUpdateAddressPath {
            list_id: self.list_id,
            person_id: self.person.id,
        }
        .to_string()
    }

    pub fn update_representative_path(&self) -> String {
        UpdateRepresentativePath {
            list_id: self.list_id,
            person_id: self.person.id,
        }
        .to_string()
    }

    pub fn delete_path(&self) -> String {
        CandidateListDeletePersonPath {
            list_id: self.list_id,
            person_id: self.person.id,
        }
        .to_string()
    }

    pub fn after_create_path(&self) -> String {
        if self.person.is_dutch() {
            CandidateListUpdateAddressPath {
                list_id: self.list_id,
                person_id: self.person.id,
            }
            .with_query_params(InitialEditQuery::default())
            .to_string()
        } else {
            UpdateRepresentativePath {
                list_id: self.list_id,
                person_id: self.person.id,
            }
            .with_query_params(InitialEditQuery::default())
            .to_string()
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(add::add_existing_person)
        .typed_post(add::add_person_to_candidate_list)
        .typed_get(update_position::update_candidate_position)
        .typed_post(update_position::update_candidate_position_submit)
        .typed_get(create::create_person_candidate_list)
        .typed_post(create::create_person_candidate_list_submit)
        .typed_get(update_address::update_person_address)
        .typed_post(update_address::update_person_address_submit)
        .typed_get(update_representative::update_representative)
        .typed_post(update_representative::update_representative_submit)
        .typed_get(update::update_person)
        .typed_post(update::update_person_submit)
        .typed_post(delete::delete_person)
}
