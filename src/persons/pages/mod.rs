use crate::{
    AppError, AppState,
    persons::{Person, PersonId},
};
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::{Deserialize, Serialize};

mod address;
mod authorised_person;
mod create;
mod delete;
mod list;
mod update;

#[derive(Serialize, Deserialize)]
pub struct InitialEditQuery {
    initial: Option<bool>,
}

impl InitialEditQuery {
    pub fn should_warn(&self) -> bool {
        !self.initial.unwrap_or(false)
    }

    pub fn new() -> Self {
        InitialEditQuery {
            initial: Some(true),
        }
    }
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons", rejection(AppError))]
pub struct PersonsPath;

#[derive(TypedPath)]
#[typed_path("/persons/new", rejection(AppError))]
pub struct PersonsNewPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons/{person_id}/edit", rejection(AppError))]
pub struct EditPersonPath {
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons/{person_id}/delete", rejection(AppError))]
pub struct DeletePersonPath {
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons/{person_id}/address", rejection(AppError))]
pub struct EditPersonAddressPath {
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons/{person_id}/authorised-person", rejection(AppError))]
pub struct EditPersonAuthorisedPersonPath {
    pub person_id: PersonId,
}

impl Person {
    pub fn list_path() -> String {
        PersonsPath {}.to_uri().to_string()
    }

    pub fn new_path() -> String {
        PersonsNewPath {}.to_uri().to_string()
    }

    pub fn edit_path(&self) -> String {
        EditPersonPath { person_id: self.id }.to_uri().to_string()
    }

    pub fn delete_path(&self) -> String {
        DeletePersonPath { person_id: self.id }.to_uri().to_string()
    }

    pub fn edit_address_path(&self) -> String {
        EditPersonAddressPath { person_id: self.id }
            .to_uri()
            .to_string()
    }

    pub fn edit_authorised_person_path(&self) -> String {
        EditPersonAuthorisedPersonPath { person_id: self.id }
            .to_uri()
            .to_string()
    }

    pub fn after_create_path(&self) -> String {
        if self.is_dutch() {
            EditPersonAddressPath { person_id: self.id }
                .with_query_params(InitialEditQuery::new())
                .to_string()
        } else {
            EditPersonAuthorisedPersonPath { person_id: self.id }
                .with_query_params(InitialEditQuery::new())
                .to_string()
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(list::list_persons)
        .typed_post(create::create_person)
        .typed_get(create::new_person_form)
        .typed_get(update::edit_person_form)
        .typed_post(update::update_person)
        .typed_get(address::edit_person_address)
        .typed_post(address::update_person_address)
        .typed_get(authorised_person::edit_authorised_person)
        .typed_post(delete::delete_person)
}
