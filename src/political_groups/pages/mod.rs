use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};

use crate::{AppError, AppState, political_groups::PoliticalGroup};

mod update;

#[derive(TypedPath)]
#[typed_path("/political-group", rejection(AppError))]
pub struct PoliticalGroupUpdatePath;

impl PoliticalGroup {
    pub fn update_path() -> String {
        PoliticalGroupUpdatePath {}.to_string()
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(update::update_political_group)
        .typed_post(update::update_political_group_submit)
}
