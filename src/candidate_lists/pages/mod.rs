use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{
    AppError, AppState,
    candidate_lists::{CandidateList, CandidateListId},
    impl_from_field,
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
#[typed_path("/candidate-lists/{id}", rejection(AppError))]
pub struct ViewCandidateListPath {
    pub id: CandidateListId,
}

impl_from_field!(ViewCandidateListPath => id: CandidateListId);

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{id}/edit", rejection(AppError))]
pub struct CandidateListsEditPath {
    pub id: CandidateListId,
}

impl_from_field!(CandidateListsEditPath => id: CandidateListId);

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{id}/delete", rejection(AppError))]
pub struct CandidateListsDeletePath {
    pub id: CandidateListId,
}

impl_from_field!(CandidateListsDeletePath => id: CandidateListId);

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{id}/reorder", rejection(AppError))]
pub struct CandidateListReorderPath {
    pub id: CandidateListId,
}

impl_from_field!(CandidateListReorderPath => id: CandidateListId);

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{id}/add", rejection(AppError))]
pub struct AddCandidatePath {
    pub id: CandidateListId,
}

impl_from_field!(AddCandidatePath => id: CandidateListId);

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{id}/new", rejection(AppError))]
pub struct CreateCandidatePath {
    pub id: CandidateListId,
}

impl_from_field!(CreateCandidatePath => id: CandidateListId);

impl CandidateList {
    pub fn list_path() -> String {
        CandidateListsPath {}.to_string()
    }

    pub fn new_path() -> String {
        CandidateListNewPath {}.to_string()
    }

    pub fn update_path(&self) -> String {
        CandidateListsEditPath { id: self.id }.to_string()
    }

    pub fn delete_path(&self) -> String {
        CandidateListsDeletePath { id: self.id }.to_string()
    }

    pub fn view_path(&self) -> String {
        ViewCandidateListPath { id: self.id }.to_string()
    }

    pub fn reorder_path(&self) -> String {
        CandidateListReorderPath { id: self.id }.to_string()
    }

    pub fn add_candidate_path(&self) -> String {
        AddCandidatePath { id: self.id }.to_string()
    }

    pub fn new_candidate_path(&self) -> String {
        CreateCandidatePath { id: self.id }.to_string()
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
