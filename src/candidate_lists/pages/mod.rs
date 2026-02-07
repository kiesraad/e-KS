use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{
    AppError, AppState, InitialEditQuery,
    candidate_lists::{CandidateList, CandidateListId},
};

mod create;
mod delete;
mod list;
mod list_submitter;
mod reorder;
mod update;
mod view;

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists", rejection(AppError))]
pub struct CandidateListsPath;

#[derive(TypedPath)]
#[typed_path("/candidate-lists/create", rejection(AppError))]
pub struct CandidateListCreatePath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{list_id}", rejection(AppError))]
pub struct ViewCandidateListPath {
    pub list_id: CandidateListId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{list_id}/update", rejection(AppError))]
pub struct CandidateListUpdatePath {
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
#[typed_path("/candidate-lists/{list_id}/list-submitter", rejection(AppError))]
pub struct UpdateListSubmitterPath {
    pub list_id: CandidateListId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/candidate-lists/{list_id}/substitute-list-submitters",
    rejection(AppError)
)]
pub struct UpdateSubstituteListSubmittersPath {
    pub list_id: CandidateListId,
}

impl CandidateList {
    pub fn list_path() -> String {
        CandidateListsPath {}.to_string()
    }

    pub fn create_path() -> String {
        CandidateListCreatePath {}.to_string()
    }

    pub fn update_path(&self) -> String {
        CandidateListUpdatePath { list_id: self.id }.to_string()
    }

    pub fn delete_path(&self) -> String {
        CandidateListsDeletePath { list_id: self.id }.to_string()
    }

    pub fn view_path(&self) -> String {
        ViewCandidateListPath { list_id: self.id }.to_string()
    }

    pub fn update_list_submitter_path(&self) -> String {
        UpdateListSubmitterPath { list_id: self.id }.to_string()
    }

    pub fn update_substitute_list_submitters_path(&self) -> String {
        UpdateSubstituteListSubmittersPath { list_id: self.id }.to_string()
    }

    pub fn reorder_path(&self) -> String {
        CandidateListReorderPath { list_id: self.id }.to_string()
    }

    pub fn add_candidate_path(&self) -> String {
        crate::candidates::AddCandidatePath { list_id: self.id }.to_string()
    }

    pub fn create_candidate_path(&self) -> String {
        crate::candidates::CreateCandidatePath { list_id: self.id }.to_string()
    }

    pub fn after_create_path(&self) -> String {
        UpdateListSubmitterPath { list_id: self.id }
            .with_query_params(InitialEditQuery::default())
            .to_string()
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        // manage lists
        .typed_get(list::list_candidate_lists)
        // create a new list
        .typed_get(create::create_candidate_list)
        .typed_post(create::create_candidate_list_submit)
        // manage single list
        .typed_get(view::view_candidate_list)
        .typed_get(update::update_candidate_list)
        .typed_post(update::update_candidate_list_submit)
        .typed_get(list_submitter::update_list_submitter)
        .typed_post(list_submitter::update_list_submitter_submit)
        .typed_get(list_submitter::update_substitute_list_submitters)
        .typed_post(list_submitter::update_substitute_list_submitters_submit)
        .typed_post(delete::delete_candidate_list)
        .typed_post(reorder::reorder_candidate_list)
}
