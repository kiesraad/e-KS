//! Typed paths for political-group routes.

use axum_extra::routing::TypedPath;

use crate::{AppError, political_groups::PoliticalGroup};

#[derive(TypedPath)]
#[typed_path("/political-group/information", rejection(AppError))]
pub struct PoliticalGroupUpdatePath;

impl PoliticalGroup {
    pub fn update_path() -> impl TypedPath {
        PoliticalGroupUpdatePath {}
    }
}
