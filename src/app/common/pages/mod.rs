use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};

mod index;
mod not_found;
mod select_election;
mod switch_election;
mod switch_locale;
mod well_known;

use crate::{AppError, AppState};

pub use not_found::not_found;

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

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(index::index)
        .typed_post(switch_locale::switch_language)
        .typed_get(switch_election::switch_election)
        .typed_post(switch_election::switch_election_submit)
        .typed_get(well_known::index)
}

/// Routes that need a session but NOT the store middleware.
/// `/select-election` must be reachable before a stream is chosen.
pub fn select_election_router() -> Router<AppState> {
    Router::new()
        .typed_get(select_election::select_election)
        .typed_post(select_election::select_election_submit)
}
