use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{
    AppError, AppState,
    candidate_lists::{CandidateList, CandidateListId},
};

mod create;
mod delete;
mod list;
mod reorder;
mod update;
mod view;

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists", rejection(AppError))]
pub struct CandidateListsPath;

#[derive(TypedPath)]
#[typed_path("/candidate-lists/new", rejection(AppError))]
pub struct CandidateListNewPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{list_id}", rejection(AppError))]
pub struct ViewCandidateListPath {
    pub list_id: CandidateListId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{list_id}/edit", rejection(AppError))]
pub struct CandidateListsEditPath {
    pub list_id: CandidateListId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{list_id}/delete", rejection(AppError))]
pub struct CandidateListsDeletePath {
    pub list_id: CandidateListId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{list_id}/reorder", rejection(AppError))]
pub struct CandidateListReorderPath {
    pub list_id: CandidateListId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{list_id}/add", rejection(AppError))]
pub struct AddCandidatePath {
    pub list_id: CandidateListId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{list_id}/new", rejection(AppError))]
pub struct CreateCandidatePath {
    pub list_id: CandidateListId,
}

impl CandidateList {
    pub fn list_path() -> String {
        CandidateListsPath {}.to_string()
    }

    pub fn new_path() -> String {
        CandidateListNewPath {}.to_string()
    }

    pub fn update_path(&self) -> String {
        CandidateListsEditPath { list_id: self.id }.to_string()
    }

    pub fn delete_path(&self) -> String {
        CandidateListsDeletePath { list_id: self.id }.to_string()
    }

    pub fn view_path(&self) -> String {
        ViewCandidateListPath { list_id: self.id }.to_string()
    }

    pub fn reorder_path(&self) -> String {
        CandidateListReorderPath { list_id: self.id }.to_string()
    }

    pub fn add_candidate_path(&self) -> String {
        AddCandidatePath { list_id: self.id }.to_string()
    }

    pub fn new_candidate_path(&self) -> String {
        CreateCandidatePath { list_id: self.id }.to_string()
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        // manage lists
        .typed_get(list::list_candidate_lists)
        .typed_get(create::new_candidate_list_form)
        .typed_post(create::create_candidate_list)
        // manage single list
        .typed_get(view::view_candidate_list)
        .typed_get(update::edit_candidate_list)
        .typed_post(update::update_candidate_list)
        .typed_post(delete::delete_candidate_list)
        .typed_post(reorder::reorder_candidate_list)
}
