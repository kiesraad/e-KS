//! Session middleware and request extraction.

use axum::{
    extract::{FromRequestParts, Request, State},
    http::request::Parts,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use chrono::Utc;

use crate::{
    AppError, AppState, Scope, Session, common::SelectElectionPath, csb::import::CsbImportPath,
};

/// Name of the session cookie used by the application.
pub const SESSION_COOKIE_NAME: &str = "EKS_SESSION_ID";

/// Builds an HTTP-only cookie that carries the session token.
pub(crate) fn build_session_cookie(session: &Session) -> Cookie<'static> {
    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, session.token().to_exposed_string());
    cookie.set_http_only(true);
    #[cfg(feature = "dev-features")]
    cookie.set_secure(false);
    #[cfg(not(feature = "dev-features"))]
    cookie.set_secure(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");

    cookie
}

/// Middleware that loads or creates a session and stores it in request extensions.
pub async fn session_middleware(
    State(state): State<AppState>,
    jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Response {
    #[cfg(feature = "dev-features")]
    if request.uri().path() == crate::auth::dev_login::DEV_LOGIN_PATH {
        return next.run(request).await;
    }

    let token = jar.get(SESSION_COOKIE_NAME).map(|cookie| cookie.value());

    let Some(mut session) = state.sessions.get_existing(token).await else {
        // redirect to home (/), which will show a login button
        return Redirect::to("/login").into_response();
    };

    session.last_activity = Utc::now();
    state.sessions.insert(session.clone()).await;

    request.extensions_mut().insert(session);

    next.run(request).await
}

/// Middleware that resolves the scoped store for the session's current election.
/// Redirects to `/select-election` when the session has not yet picked an
/// election, so the user cannot reach a route that needs a store without one.
pub async fn store_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(session) = request.extensions().get::<Session>() else {
        return next.run(request).await;
    };

    // Committee sessions never use app stores: keep them off app routes so they
    // can't create an `AppStore` in their CSB-only `(stream_id, election)`
    // partition. They belong on the CSB routes instead.
    if session.scope == Scope::CentralElectoralCommittee {
        return Redirect::to(&CsbImportPath {}.to_string()).into_response();
    }

    let (Some(stream_id), Some(election)) = (session.stream_id, session.current_election) else {
        return Redirect::to(&SelectElectionPath.to_string()).into_response();
    };

    let store = match state.store_for_stream(stream_id, election, false).await {
        Ok(store) => store,
        Err(err) => return err.into_response(),
    };

    // catch up with the latest events
    if let Err(err) = store.load().await {
        return err.into_response();
    }

    request.extensions_mut().insert(store);

    next.run(request).await
}

/// Middleware that resolves the scoped CSB store for the session's current
/// election and injects it into request extensions. Restricts CSB routes to
/// [`Scope::CentralElectoralCommittee`] sessions; other sessions are rejected.
pub async fn csb_store_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(session) = request.extensions().get::<Session>() else {
        return next.run(request).await;
    };

    // Only the central electoral committee may reach CSB routes.
    if session.scope != Scope::CentralElectoralCommittee {
        return AppError::Unauthorised.into_response();
    }

    let (Some(stream_id), Some(election)) = (session.stream_id, session.current_election) else {
        // A committee session without an election is incomplete; send it back
        // through login rather than the app's election picker.
        return Redirect::to("/login").into_response();
    };

    let store = match state.csb_store_for_stream(stream_id, election).await {
        Ok(store) => store,
        Err(err) => return err.into_response(),
    };

    // catch up with the latest events
    if let Err(err) = store.load().await {
        return err.into_response();
    }

    request.extensions_mut().insert(store);

    next.run(request).await
}

/// Extracts the current session from request extensions.
impl<S> FromRequestParts<S> for Session
where
    S: Send + Sync,
{
    type Rejection = AppError;

    /// Retrieves the session that was injected by the session middleware.
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Session>()
            .cloned()
            .ok_or(AppError::InternalServerError)
    }
}
