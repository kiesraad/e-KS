use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};

use crate::{AppError, AppState, list_submitters::ListSubmitter};

mod update;
mod view;

#[derive(TypedPath)]
#[typed_path("/political-group/list-submitter", rejection(AppError))]
pub struct ListSubmitterViewPath;

#[derive(TypedPath)]
#[typed_path("/political-group/list-submitter/update", rejection(AppError))]
pub struct ListSubmitterUpdatePath;

impl ListSubmitter {
    pub fn view_path() -> impl TypedPath {
        ListSubmitterViewPath {}
    }

    pub fn update_path() -> impl TypedPath {
        ListSubmitterUpdatePath {}
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(view::view_list_submitter)
        .typed_get(update::update_list_submitter)
        .typed_post(update::update_list_submitter_submit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_submitter_paths_match_expected_routes() {
        assert_eq!(
            ListSubmitter::view_path().to_string(),
            "/political-group/list-submitter"
        );
        assert_eq!(
            ListSubmitter::update_path().to_string(),
            "/political-group/list-submitter/update"
        );
    }

    #[test]
    fn list_submitter_router_builds() {
        let _router = router();
    }
}
