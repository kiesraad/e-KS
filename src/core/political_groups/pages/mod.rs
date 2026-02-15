use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};

use crate::{AppError, AppState, political_groups::PoliticalGroup};

mod update;

#[derive(TypedPath)]
#[typed_path("/political-group", rejection(AppError))]
pub struct PoliticalGroupUpdatePath;

impl PoliticalGroup {
    pub fn update_path() -> impl TypedPath {
        PoliticalGroupUpdatePath {}
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(update::update_political_group)
        .typed_post(update::update_political_group_submit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn political_group_update_path_matches_expected_route() {
        assert_eq!(
            PoliticalGroup::update_path().to_string(),
            "/political-group"
        );
    }

    #[test]
    fn political_group_router_builds() {
        let _router = router();
    }
}
