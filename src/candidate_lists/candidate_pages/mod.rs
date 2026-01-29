use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{
    AppError, AppState,
    candidate_lists::{Candidate, CandidateListId},
    persons::{InitialEditQuery, PersonId},
};

mod add;
mod create;
mod delete;
mod edit_address;
mod edit_authorised_person;
mod edit_position;
mod update;

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{list_id}/reorder/{person_id}", rejection(AppError))]
pub struct EditCandidatePositionPath {
    pub list_id: CandidateListId,
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/candidate-lists/{list_id}/authorised-person/{person_id}",
    rejection(AppError)
)]
pub struct EditAuthorisedPersonPath {
    pub list_id: CandidateListId,
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{list_id}/edit/{person_id}", rejection(AppError))]
pub struct CandidateListEditPersonPath {
    pub list_id: CandidateListId,
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{list_id}/address/{person_id}", rejection(AppError))]
pub struct CandidateListEditAddressPath {
    pub list_id: CandidateListId,
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{list_id}/delete/{person_id}", rejection(AppError))]
pub struct CandidateListDeletePersonPath {
    pub list_id: CandidateListId,
    pub person_id: PersonId,
}

impl Candidate {
    pub fn edit_position_path(&self) -> String {
        EditCandidatePositionPath {
            list_id: self.list_id,
            person_id: self.person.id,
        }
        .to_string()
    }

    pub fn edit_path(&self) -> String {
        CandidateListEditPersonPath {
            list_id: self.list_id,
            person_id: self.person.id,
        }
        .to_string()
    }

    pub fn edit_address_path(&self) -> String {
        CandidateListEditAddressPath {
            list_id: self.list_id,
            person_id: self.person.id,
        }
        .to_string()
    }

    pub fn edit_authorised_person_path(&self) -> String {
        EditAuthorisedPersonPath {
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
            CandidateListEditAddressPath {
                list_id: self.list_id,
                person_id: self.person.id,
            }
            .with_query_params(InitialEditQuery::new())
            .to_string()
        } else {
            EditAuthorisedPersonPath {
                list_id: self.list_id,
                person_id: self.person.id,
            }
            .with_query_params(InitialEditQuery::new())
            .to_string()
        }
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
        .typed_get(edit_authorised_person::edit_authorised_person)
        .typed_get(update::edit_person_form)
        .typed_post(update::update_person)
        .typed_post(delete::delete_person)
}
