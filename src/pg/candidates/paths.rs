//! Typed paths for candidate routes and path helpers on [`Candidate`].

use axum_extra::routing::TypedPath;
use serde::Deserialize;

use crate::{
    AppError, QueryParamState,
    structs::{candidate_lists::CandidateListId, candidates::Candidate, persons::PersonId},
};

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
    pub fn update_position_path(&self) -> impl TypedPath {
        UpdateCandidatePositionPath {
            list_id: self.list_id,
            person_id: self.person.id,
        }
    }

    pub fn update_path(&self) -> impl TypedPath {
        CandidateListUpdatePersonPath {
            list_id: self.list_id,
            person_id: self.person.id,
        }
    }

    pub fn update_address_path(&self) -> impl TypedPath {
        CandidateListUpdateAddressPath {
            list_id: self.list_id,
            person_id: self.person.id,
        }
    }

    pub fn update_representative_path(&self) -> impl TypedPath {
        UpdateRepresentativePath {
            list_id: self.list_id,
            person_id: self.person.id,
        }
    }

    pub fn delete_path(&self) -> impl TypedPath {
        CandidateListDeletePersonPath {
            list_id: self.list_id,
            person_id: self.person.id,
        }
    }

    pub fn after_update_path(&self) -> String {
        if self.person.needs_representative() {
            self.update_representative_path().to_string()
        } else {
            self.update_address_path().to_string()
        }
    }

    pub fn after_create_path(&self) -> String {
        if !self.person.needs_representative() {
            CandidateListUpdateAddressPath {
                list_id: self.list_id,
                person_id: self.person.id,
            }
            .with_query_params(QueryParamState::created())
            .to_string()
        } else {
            UpdateRepresentativePath {
                list_id: self.list_id,
                person_id: self.person.id,
            }
            .with_query_params(QueryParamState::created())
            .to_string()
        }
    }
}
