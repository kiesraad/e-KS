use crate::{
    AppError, AppState, InitialEditQuery,
    persons::{Person, PersonId},
};
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

mod address;
mod create;
mod delete;
mod list;
mod representative;
mod update;

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons", rejection(AppError))]
pub struct PersonsPath;

#[derive(TypedPath)]
#[typed_path("/persons/create", rejection(AppError))]
pub struct PersonsCreatePath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons/{person_id}/update", rejection(AppError))]
pub struct UpdatePersonPath {
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons/{person_id}/delete", rejection(AppError))]
pub struct DeletePersonPath {
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons/{person_id}/address", rejection(AppError))]
pub struct UpdatePersonAddressPath {
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons/{person_id}/representative", rejection(AppError))]
pub struct UpdateRepresentativePath {
    pub person_id: PersonId,
}

impl Person {
    pub fn list_path() -> String {
        PersonsPath {}.to_uri().to_string()
    }

    pub fn create_path() -> String {
        PersonsCreatePath {}.to_uri().to_string()
    }

    pub fn update_path(&self) -> String {
        UpdatePersonPath { person_id: self.id }.to_uri().to_string()
    }

    pub fn delete_path(&self) -> String {
        DeletePersonPath { person_id: self.id }.to_uri().to_string()
    }

    pub fn update_address_path(&self) -> String {
        UpdatePersonAddressPath { person_id: self.id }
            .to_uri()
            .to_string()
    }

    pub fn update_representative_path(&self) -> String {
        UpdateRepresentativePath { person_id: self.id }
            .to_uri()
            .to_string()
    }

    pub fn after_create_path(&self) -> String {
        if self.is_dutch() {
            UpdatePersonAddressPath { person_id: self.id }
                .with_query_params(InitialEditQuery::default())
                .to_string()
        } else {
            UpdateRepresentativePath { person_id: self.id }
                .with_query_params(InitialEditQuery::default())
                .to_string()
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(list::list_persons)
        .typed_post(create::create_person_submit)
        .typed_get(create::create_person)
        .typed_get(update::update_person)
        .typed_post(update::update_person_submit)
        .typed_get(address::update_person_address)
        .typed_post(address::update_person_address_submit)
        .typed_get(representative::update_representative)
        .typed_post(representative::update_representative_submit)
        .typed_post(delete::delete_person)
}
