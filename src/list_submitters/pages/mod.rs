use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{
    AppError, AppState,
    list_submitters::{ListSubmitter, ListSubmitterId},
};

mod list_submitter_create;
mod list_submitter_delete;
mod list_submitter_update;
mod list_submitters;

#[derive(TypedPath, Deserialize)]
#[typed_path("/political-group/list-submitters", rejection(AppError))]
pub struct ListSubmittersPath;

#[derive(TypedPath)]
#[typed_path("/political-group/list-submitters/create", rejection(AppError))]
pub struct ListSubmitterCreatePath;

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/political-group/list-submitters/{submitter_id}/update",
    rejection(AppError)
)]
pub struct ListSubmitterUpdatePath {
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

impl ListSubmitter {
    pub fn list_path() -> String {
        ListSubmittersPath {}.to_string()
    }

    pub fn create_path() -> String {
        ListSubmitterCreatePath {}.to_string()
    }

    pub fn update_path(&self) -> String {
        ListSubmitterUpdatePath {
            submitter_id: self.id,
        }
        .to_string()
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
        .typed_get(list_submitters::list_submitters)
        .typed_get(list_submitter_create::create_list_submitter)
        .typed_post(list_submitter_create::create_list_submitter_submit)
        .typed_get(list_submitter_update::update_list_submitter)
        .typed_post(list_submitter_update::update_list_submitter_submit)
        .typed_post(list_submitter_delete::delete_list_submitter)
}
