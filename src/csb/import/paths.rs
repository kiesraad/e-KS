//! Typed paths for the CSB import route.

use axum_extra::routing::TypedPath;

use crate::AppError;

#[derive(TypedPath)]
#[typed_path("/csb/import", rejection(AppError))]
pub struct CsbImportPath;

#[derive(TypedPath)]
#[typed_path("/csb/create-empty", rejection(AppError))]
pub struct CsbCreateEmptyPath;
