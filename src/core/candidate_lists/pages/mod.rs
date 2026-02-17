use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{
    AppError, AppState, QueryParamState,
    candidate_lists::{CandidateList, CandidateListId},
    persons::PersonId,
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
    pub fn list_path() -> impl TypedPath {
        CandidateListsPath {}
    }

    pub fn highlight_path(&self, person_id: PersonId) -> impl TypedPath {
        ViewCandidateListPath { list_id: self.id }
            .with_query_params(QueryParamState::highlight(person_id.into()))
    }

    pub fn highlight_success_path(&self, person_id: PersonId) -> impl TypedPath {
        ViewCandidateListPath { list_id: self.id }
            .with_query_params(QueryParamState::highlight_success(person_id.into()))
    }

    pub fn create_path() -> impl TypedPath {
        CandidateListCreatePath {}
    }

    pub fn update_path(&self) -> impl TypedPath {
        CandidateListUpdatePath { list_id: self.id }
    }

    pub fn delete_path(&self) -> impl TypedPath {
        CandidateListsDeletePath { list_id: self.id }
    }

    pub fn view_path(&self) -> impl TypedPath {
        ViewCandidateListPath { list_id: self.id }
    }

    pub fn update_list_submitter_path(&self) -> impl TypedPath {
        UpdateListSubmitterPath { list_id: self.id }
    }

    pub fn update_substitute_list_submitters_path(&self) -> impl TypedPath {
        UpdateSubstituteListSubmittersPath { list_id: self.id }
    }

    pub fn reorder_path(&self) -> impl TypedPath {
        CandidateListReorderPath { list_id: self.id }
    }

    pub fn add_candidate_path(&self) -> impl TypedPath {
        crate::candidates::AddCandidatePath { list_id: self.id }
    }

    pub fn create_candidate_path(&self) -> impl TypedPath {
        crate::candidates::CreateCandidatePath { list_id: self.id }
    }

    pub fn after_create_path(&self) -> impl TypedPath {
        UpdateListSubmitterPath { list_id: self.id }.with_query_params(QueryParamState::created())
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
        .typed_post(delete::delete_candidate_list)
        .typed_post(reorder::reorder_candidate_list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{candidate_lists::CandidateListId, test_utils::sample_candidate_list};

    #[test]
    fn candidate_list_paths_match_expected_routes() {
        let list = sample_candidate_list(CandidateListId::new());

        assert_eq!(CandidateList::list_path().to_string(), "/candidate-lists");
        assert_eq!(
            CandidateList::create_path().to_string(),
            "/candidate-lists/create"
        );
        assert_eq!(
            list.update_path().to_string(),
            format!("/candidate-lists/{}/update", list.id)
        );
        assert_eq!(
            list.delete_path().to_string(),
            format!("/candidate-lists/{}/delete", list.id)
        );
        assert_eq!(
            list.view_path().to_string(),
            format!("/candidate-lists/{}", list.id)
        );
        assert_eq!(
            list.update_list_submitter_path().to_string(),
            format!("/candidate-lists/{}/list-submitter", list.id)
        );
        assert_eq!(
            list.update_substitute_list_submitters_path().to_string(),
            format!("/candidate-lists/{}/substitute-list-submitters", list.id)
        );
        assert_eq!(
            list.reorder_path().to_string(),
            format!("/candidate-lists/{}/reorder", list.id)
        );
        assert_eq!(
            list.add_candidate_path().to_string(),
            format!("/candidate-lists/{}/add", list.id)
        );
        assert_eq!(
            list.create_candidate_path().to_string(),
            format!("/candidate-lists/{}/create", list.id)
        );
    }

    #[test]
    fn candidate_list_after_create_path_includes_initial_query() {
        let list = sample_candidate_list(CandidateListId::new());
        let expected = format!(
            "/candidate-lists/{}/list-submitter?&initial=true&success=true",
            list.id
        );

        assert_eq!(list.after_create_path().to_string(), expected);
    }

    #[test]
    fn candidate_list_router_builds() {
        let _router = router();
    }
}
