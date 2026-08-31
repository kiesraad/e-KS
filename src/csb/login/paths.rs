//! Typed paths for the CSB GitHub login flow.

use axum_extra::routing::TypedPath;

use crate::AppError;

#[derive(TypedPath)]
#[typed_path("/csb/login", rejection(AppError))]
pub struct CsbLoginPath;

/// Starts the OAuth round-trip. A plain link, so the browser navigates to
/// GitHub without a form submission: `form-action` never enters the picture
/// and the app keeps one strict Content-Security-Policy for every page.
#[derive(TypedPath)]
#[typed_path("/csb/login/start", rejection(AppError))]
pub struct CsbLoginStartPath;

#[derive(TypedPath)]
#[typed_path("/csb/login/callback", rejection(AppError))]
pub struct CsbLoginCallbackPath;
