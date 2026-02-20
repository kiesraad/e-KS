use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{
    AppError, AppState,
    substitute_list_submitters::{SubstituteSubmitter, SubstituteSubmitterId},
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
    pub fn create_path() -> impl TypedPath {
        SubstituteSubmitterCreatePath {}
    }

    pub fn update_path(&self) -> impl TypedPath {
        SubstituteSubmitterUpdatePath {
            sub_submitter_id: self.id,
        }
    }

    pub fn delete_path(&self) -> impl TypedPath {
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
    use crate::{
        substitute_list_submitters::SubstituteSubmitterId, test_utils::sample_substitute_submitter,
    };

    #[test]
    fn substitute_submitter_paths_match_expected_routes() {
        let submitter = sample_substitute_submitter(SubstituteSubmitterId::new());

        assert_eq!(
            SubstituteSubmitter::create_path().to_string(),
            "/political-group/substitute-submitters/create"
        );
        assert_eq!(
            submitter.update_path().to_string(),
            format!(
                "/political-group/substitute-submitters/{}/update",
                submitter.id
            )
        );
        assert_eq!(
            submitter.delete_path().to_string(),
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
