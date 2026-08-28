//! Typed paths for the CSB GitHub login flow.

use axum_extra::routing::TypedPath;

use crate::AppError;

#[derive(TypedPath)]
#[typed_path("/csb/login", rejection(AppError))]
pub struct CsbLoginPath;

#[derive(TypedPath)]
#[typed_path("/csb/login/callback", rejection(AppError))]
pub struct CsbLoginCallbackPath;
