use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};

use crate::{AppError, AppState, list_designation::ListDesignation};

mod update;

#[derive(TypedPath)]
#[typed_path("/political-group", rejection(AppError))]
pub struct ListDesignationUpdatePath;

impl ListDesignation {
    pub fn update_path() -> impl TypedPath {
        ListDesignationUpdatePath {}
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(update::update_list_designation)
        .typed_post(update::update_list_designation_submit)
}
