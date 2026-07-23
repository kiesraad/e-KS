//! Typed paths for the common (session-level) routes.

use axum_extra::routing::TypedPath;

use crate::AppError;

#[derive(TypedPath)]
#[typed_path("/login", rejection(AppError))]
pub struct LoginStartPath;

#[derive(TypedPath)]
#[typed_path("/", rejection(AppError))]
pub struct IndexPath;

#[derive(TypedPath)]
#[typed_path("/language", rejection(AppError))]
pub struct SwitchLanguagePath;

#[derive(TypedPath)]
#[typed_path("/switch-election", rejection(AppError))]
pub struct SwitchElectionPath;

#[derive(TypedPath)]
#[typed_path("/select-election", rejection(AppError))]
pub struct SelectElectionPath;

#[derive(TypedPath)]
#[typed_path("/hide-download-warning", rejection(AppError))]
pub struct HideDownloadWarningPath;

#[derive(TypedPath)]
#[typed_path("/logout", rejection(AppError))]
pub struct LogoutPath;

#[derive(TypedPath)]
#[typed_path("/logged-out", rejection(AppError))]
pub struct LoggedOutPath;
