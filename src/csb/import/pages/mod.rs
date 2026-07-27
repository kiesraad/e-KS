use axum::Router;
use axum_extra::routing::RouterExt;

use crate::AppState;

pub(crate) use super::paths::CsbImportPath;

mod import;

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(import::import)
        .typed_post(import::import_submit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csb_import_path_matches_expected_route() {
        assert_eq!(CsbImportPath {}.to_string(), "/csb/import");
    }

    #[test]
    fn csb_import_router_builds() {
        let _router = router();
    }
}
