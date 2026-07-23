use axum::Router;
use axum_extra::routing::RouterExt;

use crate::AppState;

mod update;

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(update::update_political_group)
        .typed_post(update::update_political_group_submit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::political_groups::PoliticalGroup;

    #[test]
    fn political_group_update_path_matches_expected_route() {
        assert_eq!(
            PoliticalGroup::update_path().to_string(),
            "/political-group/information"
        );
    }

    #[test]
    fn political_group_router_builds() {
        let _router = router();
    }
}
