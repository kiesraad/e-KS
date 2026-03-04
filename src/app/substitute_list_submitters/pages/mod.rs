use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{
    AppError, AppState,
    list_submitters::{ListSubmitter, ListSubmitterId},
};

mod create;
mod delete;
mod update;

#[derive(TypedPath)]
#[typed_path("/political-group/substitute-submitters/create", rejection(AppError))]
pub struct SubstituteSubmitterCreatePath;

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/political-group/substitute-submitters/{sub_submitter_id}/update",
    rejection(AppError)
)]
pub struct SubstituteSubmitterUpdatePath {
    pub sub_submitter_id: ListSubmitterId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/political-group/substitute-submitters/{sub_submitter_id}/delete",
    rejection(AppError)
)]
pub struct SubstituteSubmitterDeletePath {
    pub sub_submitter_id: ListSubmitterId,
}

impl ListSubmitter {
    pub fn substitute_create_path() -> impl TypedPath {
        SubstituteSubmitterCreatePath {}
    }

    pub fn substitute_update_path(&self) -> impl TypedPath {
        SubstituteSubmitterUpdatePath {
            sub_submitter_id: self.id,
        }
    }

    pub fn substitute_delete_path(&self) -> impl TypedPath {
        SubstituteSubmitterDeletePath {
            sub_submitter_id: self.id,
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(create::create_substitute_submitter)
        .typed_post(create::create_substitute_submitter_submit)
        .typed_get(update::update_substitute_submitter)
        .typed_post(update::update_substitute_submitter_submit)
        .typed_post(delete::delete_substitute_submitter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::sample_list_submitter;

    #[test]
    fn substitute_submitter_paths_match_expected_routes() {
        let submitter = sample_list_submitter(ListSubmitterId::new());

        assert_eq!(
            ListSubmitter::substitute_create_path().to_string(),
            "/political-group/substitute-submitters/create"
        );
        assert_eq!(
            submitter.substitute_update_path().to_string(),
            format!(
                "/political-group/substitute-submitters/{}/update",
                submitter.id
            )
        );
        assert_eq!(
            submitter.substitute_delete_path().to_string(),
            format!(
                "/political-group/substitute-submitters/{}/delete",
                submitter.id
            )
        );
    }

    #[test]
    fn substitute_submitter_router_builds() {
        let _router = router();
    }
}
