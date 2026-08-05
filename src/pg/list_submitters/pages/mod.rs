use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};

use crate::{AppError, AppRequestState, QueryParamState, structs::list_submitters::ListSubmitter};

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

    pub fn update_path_from(from: impl std::fmt::Display) -> impl TypedPath {
        ListSubmitterUpdatePath {}.with_query_params(QueryParamState::redirect_to(from.to_string()))
    }
}

pub fn router<S: AppRequestState>() -> Router<S> {
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
        let _router = router::<crate::AppState>();
    }
}
