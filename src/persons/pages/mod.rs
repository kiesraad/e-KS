use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppError, AppState, Locale,
    pagination::Pagination,
    persons::structs::{Person, PersonSort},
    t,
};

mod address;
mod create;
mod delete;
mod list;
mod update;

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons", rejection(AppError))]
pub(crate) struct PersonsPath;

#[derive(TypedPath)]
#[typed_path("/persons/new", rejection(AppError))]
pub(crate) struct PersonsNewPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons/{id}/edit", rejection(AppError))]
pub(crate) struct EditPersonPath {
    pub(crate) id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons/{id}/delete", rejection(AppError))]
pub(crate) struct DeletePersonPath {
    pub(crate) id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons/{id}/address", rejection(AppError))]
pub(crate) struct EditPersonAddressPath {
    pub(crate) id: Uuid,
}

impl Person {
    pub fn list_path() -> String {
        PersonsPath {}.to_uri().to_string()
    }

    pub fn list_path_with_pagination(pagination: &Pagination<PersonSort>) -> String {
        format!("{}{}", PersonsPath {}.to_uri(), pagination.as_query())
    }

    pub fn new_path() -> String {
        PersonsNewPath {}.to_uri().to_string()
    }

    pub fn edit_path(&self) -> String {
        EditPersonPath { id: self.id }.to_uri().to_string()
    }

    pub fn edit_address_path(&self) -> String {
        EditPersonAddressPath { id: self.id }.to_uri().to_string()
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

pub fn person_not_found(id: Uuid, locale: Locale) -> AppError {
    AppError::NotFound(t!("person.not_found", &locale, id))
}
