//! Placeholder auth-service: the public API surface only.
//!
pub mod error;
pub mod pending;
pub mod state;

mod handlers;

use axum::{
    Router,
    extract::FromRef,
    routing::{get, post},
};

pub use crate::{
    handlers::{handle_login, handle_logout},
    pending::{PENDING_REQUEST_TTL, PendingRequests},
    state::{AuthFailure, AuthServiceState, AuthState, SubjectId},
};

/// Build the SAML SP router for the protocol endpoints (metadata, ACS, SLS).
pub fn router<S>() -> Router<S>
where
    S: AuthState,
    AuthServiceState: FromRef<S>,
{
    Router::new()
        .route("/saml/sp/metadata", get(handlers::handle_metadata))
        .route("/saml/sp/acs", get(handlers::handle_acs))
        .route("/saml/sp/logout", post(handlers::handle_sls))
}
