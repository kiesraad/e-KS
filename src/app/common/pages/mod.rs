use axum::{Router, routing::get};
use axum_extra::routing::{RouterExt, TypedPath};

mod auth;
mod hide_download_warning;
mod index;
mod not_found;
mod select_election;
mod switch_election;
mod switch_locale;
mod well_known;

use crate::{AppError, AppState};

pub use auth::auth_failure_response;
pub use not_found::not_found;

#[derive(TypedPath)]
#[typed_path("/login", rejection(AppError))]
pub struct LoginStartPath;

#[derive(TypedPath)]
#[typed_path("/logout", rejection(AppError))]
pub struct LogoutPath;

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

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(index::index)
        .typed_get(switch_election::switch_election)
        .typed_post(switch_election::switch_election_submit)
        .typed_post(hide_download_warning::hide_download_warning)
}

/// Routes that need a session but NOT the store middleware.
///
/// `/select-election` must be reachable before a stream is chosen, and
/// `/language` must be reachable by every session, including committee (CSB)
/// sessions, which `store_middleware` redirects away from app routes. Switching
/// the locale only touches the session, so it belongs here rather than behind
/// the store.
pub fn session_only_router() -> Router<AppState> {
    Router::new()
        .typed_post(switch_locale::switch_language)
        .typed_get(select_election::select_election)
        .typed_post(select_election::select_election_submit)
}

/// Routes for paths under .well-known
pub fn wellknown_router() -> Router<AppState> {
    Router::new().typed_get(well_known::security_txt)
}

/// Routes mounted outside the session middleware (no session required):
/// - `/login`: GET shows the DigiD start page, POST starts SAML SSO
/// - `/logout`: GET initiates logout or shows the post-logout confirmation page (TVS T7)
pub fn public_router() -> Router<AppState> {
    Router::new()
        .route(
            "/login",
            get(auth::login_start).post(auth_service::handle_login::<AppState>),
        )
        .route("/logout", get(auth::logout))
}
