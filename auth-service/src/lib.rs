//! Placeholder auth-service: the public API surface only.
//!
pub mod error;
pub mod pending;
pub mod state;

mod handlers;

use axum::{Router, extract::FromRef};
use axum_extra::routing::{RouterExt, TypedPath};

pub use crate::{
    handlers::{handle_login, handle_logout},
    pending::{PENDING_REQUEST_TTL, PendingRequests},
    state::{AuthFailure, AuthServiceState, AuthState, SubjectId},
};

/// SP metadata endpoint.
#[derive(TypedPath)]
#[typed_path("/saml/sp/metadata")]
pub struct SamlMetadataPath;

/// Assertion Consumer Service endpoint.
#[derive(TypedPath)]
#[typed_path("/saml/sp/acs")]
pub struct SamlAcsPath;

/// Single-logout (SLS) endpoint.
#[derive(TypedPath)]
#[typed_path("/saml/sp/logout")]
pub struct SamlLogoutPath;

/// Build the SAML SP router for the protocol endpoints (metadata, ACS, SLS).
pub fn router<S>() -> Router<S>
where
    S: AuthState,
    AuthServiceState: FromRef<S>,
{
    Router::new()
        .typed_get(handlers::handle_metadata)
        .typed_get(handlers::handle_acs)
        .typed_post(handlers::handle_sls)
}
