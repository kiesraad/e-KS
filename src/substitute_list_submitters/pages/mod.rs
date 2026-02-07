use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{
    AppError, AppState,
    substitute_list_submitters::{SubstituteSubmitter, SubstituteSubmitterId},
};

mod substitute_submitter_create;
mod substitute_submitter_delete;
mod substitute_submitter_update;

#[derive(TypedPath)]
#[typed_path("/political-group/substitute-submitters/create", rejection(AppError))]
pub struct SubstituteSubmitterCreatePath;

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/political-group/substitute-submitters/{sub_submitter_id}/update",
    rejection(AppError)
)]
pub struct SubstituteSubmitterUpdatePath {
    pub sub_submitter_id: SubstituteSubmitterId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/political-group/substitute-submitters/{sub_submitter_id}/delete",
    rejection(AppError)
)]
pub struct SubstituteSubmitterDeletePath {
    pub sub_submitter_id: SubstituteSubmitterId,
}

impl SubstituteSubmitter {
    pub fn create_path() -> String {
        SubstituteSubmitterCreatePath {}.to_string()
    }

    pub fn update_path(&self) -> String {
        SubstituteSubmitterUpdatePath {
            sub_submitter_id: self.id,
        }
        .to_string()
    }

    pub fn delete_path(&self) -> String {
        SubstituteSubmitterDeletePath {
            sub_submitter_id: self.id,
        }
        .to_string()
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(substitute_submitter_create::create_substitute_submitter)
        .typed_post(substitute_submitter_create::create_substitute_submitter_submit)
        .typed_get(substitute_submitter_update::update_substitute_submitter)
        .typed_post(substitute_submitter_update::update_substitute_submitter_submit)
        .typed_post(substitute_submitter_delete::delete_substitute_submitter)
}
