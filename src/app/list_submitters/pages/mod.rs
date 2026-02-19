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
    pub fn list_path() -> impl TypedPath {
        ListSubmittersPath {}
    }

    pub fn create_path() -> impl TypedPath {
        ListSubmitterCreatePath {}
    }

    pub fn update_path(&self) -> impl TypedPath {
        ListSubmitterUpdatePath {
            submitter_id: self.id,
        }
    }

    pub fn delete_path(&self) -> impl TypedPath {
        ListSubmitterDeletePath {
            submitter_id: self.id,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{list_submitters::ListSubmitterId, test_utils::sample_list_submitter};

    #[test]
    fn list_submitter_paths_match_expected_routes() {
        let submitter = sample_list_submitter(ListSubmitterId::new());

        assert_eq!(
            ListSubmitter::list_path().to_string(),
            "/political-group/list-submitters"
        );
        assert_eq!(
            ListSubmitter::create_path().to_string(),
            "/political-group/list-submitters/create"
        );
        assert_eq!(
            submitter.update_path().to_string(),
            format!("/political-group/list-submitters/{}/update", submitter.id)
        );
        assert_eq!(
            submitter.delete_path().to_string(),
            format!("/political-group/list-submitters/{}/delete", submitter.id)
        );
    }

    #[test]
    fn list_submitter_router_builds() {
        let _router = router();
    }
}
