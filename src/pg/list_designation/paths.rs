//! Typed paths for list-designation routes.

use axum_extra::routing::TypedPath;

use crate::{AppError, list_designation::ListDesignation};

#[derive(TypedPath)]
#[typed_path("/political-group", rejection(AppError))]
pub struct ListDesignationUpdatePath;

impl ListDesignation {
    pub fn update_path() -> impl TypedPath {
        ListDesignationUpdatePath {}
    }
}
