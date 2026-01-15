use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    AppError, AppState, Locale,
    candidate_lists::{
        self,
        structs::{CandidateList, FullCandidateList},
    },
    t,
};

mod create;
mod delete;
mod list;
mod reorder;
mod update;
mod view;

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists", rejection(AppError))]
pub(crate) struct CandidateListsPath;

#[derive(TypedPath)]
#[typed_path("/candidate-lists/new", rejection(AppError))]
pub(crate) struct CandidateListNewPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{id}", rejection(AppError))]
pub(crate) struct ViewCandidateListPath {
    pub(crate) id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{id}/edit", rejection(AppError))]
pub(crate) struct CandidateListsEditPath {
    pub(crate) id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{id}/delete", rejection(AppError))]
pub(crate) struct CandidateListsDeletePath {
    pub(crate) id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{id}/reorder", rejection(AppError))]
pub(crate) struct CandidateListReorderPath {
    pub(crate) id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{id}/add", rejection(AppError))]
pub(crate) struct AddCandidatePath {
    pub(crate) id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/candidate-lists/{candidate_list}/new", rejection(AppError))]
pub(crate) struct CreateCandidatePath {
    pub(crate) candidate_list: Uuid,
}

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
        CreateCandidatePath {
            candidate_list: self.id,
        }
        .to_string()
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

pub(crate) fn candidate_list_not_found(id: Uuid, locale: Locale) -> AppError {
    AppError::NotFound(t!("candidate_list.not_found", &locale, id))
}

pub(super) async fn load_candidate_list(
    conn: &mut PgConnection,
    id: &Uuid,
    locale: Locale,
) -> Result<FullCandidateList, AppError> {
    candidate_lists::repository::get_full_candidate_list(conn, id)
        .await?
        .ok_or_else(|| candidate_list_not_found(*id, locale))
}
