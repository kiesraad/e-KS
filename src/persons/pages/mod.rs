use crate::{
    AppError, AppState,
    persons::{Person, PersonId},
};
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

mod address;
mod create;
mod delete;
mod list;
mod update;

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

    pub fn edit_address_path(&self) -> String {
        EditPersonAddressPath { person_id: self.id }
            .to_uri()
            .to_string()
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
        .typed_post(delete::delete_person)
}
