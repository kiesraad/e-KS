use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{
    AppError, AppState,
    political_groups::{ListSubmitter, ListSubmitterId, PoliticalGroup},
};

mod list_submitter_create;
mod list_submitter_delete;
mod list_submitter_update;
mod list_submitters;
mod update;

#[derive(TypedPath, Deserialize)]
#[typed_path("/political-group", rejection(AppError))]
pub struct PoliticalGroupEditPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/political-group/list-submitters", rejection(AppError))]
pub struct ListSubmittersPath;

#[derive(TypedPath)]
#[typed_path("/political-group/list-submitters/new", rejection(AppError))]
pub struct ListSubmitterNewPath;

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/political-group/list-submitters/{submitter_id}/edit",
    rejection(AppError)
)]
pub struct ListSubmitterEditPath {
    pub submitter_id: ListSubmitterId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/political-group/list-submitters/{submitter_id}/delete",
    rejection(AppError)
)]
pub struct ListSubmitterDeletePath {
    pub submitter_id: ListSubmitterId,
}

impl PoliticalGroup {
    pub fn edit_path() -> String {
        PoliticalGroupEditPath {}.to_string()
    }
}

impl ListSubmitter {
    pub fn list_path() -> String {
        ListSubmittersPath {}.to_string()
    }

    pub fn new_path() -> String {
        ListSubmitterNewPath {}.to_string()
    }

    pub fn edit_path(&self) -> String {
        ListSubmitterEditPath {
            submitter_id: self.id,
        }
        .to_string()
    }

    pub fn select_default_path() -> String {
        ListSubmittersPath {}.to_string()
    }

    pub fn delete_path(&self) -> String {
        ListSubmitterDeletePath {
            submitter_id: self.id,
        }
        .to_string()
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(update::edit_political_group)
        .typed_post(update::update_political_group)
        .typed_get(list_submitters::list_submitters)
        .typed_post(list_submitters::update_list_submitters)
        .typed_get(list_submitter_create::new_list_submitter_form)
        .typed_post(list_submitter_create::create_list_submitter)
        .typed_get(list_submitter_update::edit_list_submitter)
        .typed_post(list_submitter_update::update_list_submitter)
        .typed_post(list_submitter_delete::delete_list_submitter)
}
