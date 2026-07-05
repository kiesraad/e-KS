//! Session and store-loading middleware.

use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use serde::{Serialize, de::DeserializeOwned};

use super::maintenance::handle_db_error;
use crate::{
    AppError, AppState, SESSION_COOKIE_NAME, Scope, Session,
    auth::session_extractor::user_agent_hash,
    common::{LoginStartPath, SelectElectionPath},
    csb::examination::CsbExaminationOverviewPath,
    store::{Store, StoreData},
};

/// Middleware that loads or creates a session and stores it in request extensions.
pub async fn session_middleware(
    State(state): State<AppState>,
    jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Response {
    #[cfg(feature = "dev-features")]
    if request.uri().path() == super::dev_login::DEV_LOGIN_PATH {
        return next.run(request).await;
    }

    let token = jar.get(SESSION_COOKIE_NAME).map(|cookie| cookie.value());

    let mut session = match state.sessions.get_existing(token).await {
        Ok(Some(session)) => session,
        // Send unauthenticated users to the login start page (DigiD button +
        // explanation), not straight into the SAML flow at `/login`.
        Ok(None) => return Redirect::to(&LoginStartPath.to_string()).into_response(),
        // A database error here must not masquerade as "logged out": trip the
        // maintenance gate instead of redirecting to login.
        Err(err) => return handle_db_error(&state.db_health, err, &request),
    };

    if let Some(rejection) =
        reject_on_user_agent_mismatch(&state, &session, request.headers(), token).await
    {
        return rejection;
    }

    session.last_activity = Utc::now();
    state.sessions.insert(session.clone()).await;

    request.extensions_mut().insert(session);

    next.run(request).await
}

/// User-agent pinning: reject (and drop) a session replayed from a different
/// client. Only enforced when the session recorded a UA; returns the login
/// redirect to short-circuit the middleware, or `None` when the UA matches.
async fn reject_on_user_agent_mismatch(
    state: &AppState,
    session: &Session,
    headers: &HeaderMap,
    token: Option<&str>,
) -> Option<Response> {
    let expected = session.user_agent_hash.as_deref()?;
    if user_agent_hash(headers) == expected {
        return None;
    }

    tracing::warn!("session user-agent mismatch; dropping session");
    if let Some(token) = token {
        state.sessions.remove(token).await;
    }
    Some(Redirect::to(&LoginStartPath.to_string()).into_response())
}

/// Middleware that resolves the scoped store for the session's current election.
/// Redirects to `/select-election` when the session has not yet picked an
/// election, so the user cannot reach a route that needs a store without one.
pub async fn store_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let Some(session) = request.extensions().get::<Session>() else {
        return next.run(request).await;
    };

    // Committee sessions never use app stores: keep them off app routes so they
    // can't create an `AppStore` in their CSB-only `(stream_id, election)`
    // partition. They belong on the CSB routes instead.
    if session.scope == Scope::CentralElectoralCommittee {
        return Redirect::to(&CsbExaminationOverviewPath {}.to_string()).into_response();
    }

    let (Some(stream_id), Some(election)) = (session.stream_id, session.current_election) else {
        return Redirect::to(&SelectElectionPath.to_string()).into_response();
    };

    let resolved = state.store_for_stream(stream_id, election, false).await;
    inject_loaded_store(&state, resolved, request, next).await
}

/// Middleware that loads the global CSB main store for the session's current
/// election and injects it into request extensions. Restricts CSB routes to
/// [`Scope::CentralElectoralCommittee`] sessions; other sessions are rejected.
pub async fn csb_store_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let Some(session) = request.extensions().get::<Session>() else {
        return next.run(request).await;
    };

    // Only the central electoral committee may reach CSB routes.
    if session.scope != Scope::CentralElectoralCommittee {
        return AppError::Unauthorised.into_response();
    }

    let Some(election) = session.current_election else {
        // A committee session without an election is incomplete; send it back
        // through login rather than the app's election picker.
        return Redirect::to("/login").into_response();
    };

    let resolved = state.csb_main_store(election).await;
    inject_loaded_store(&state, resolved, request, next).await
}

/// Replay a resolved store's latest events, inject it into the request, and
/// continue down the chain. Shared by [`store_middleware`] and
/// [`csb_store_middleware`]: a failure to resolve or replay is an
/// infrastructure problem, so it trips the maintenance gate rather than
/// serving a stale or missing store.
async fn inject_loaded_store<D>(
    state: &AppState,
    resolved: Result<Store<D>, AppError>,
    mut request: Request,
    next: Next,
) -> Response
where
    D: StoreData,
    D::Event: Serialize + DeserializeOwned,
{
    let store = match resolved {
        Ok(store) => store,
        Err(err) => return handle_db_error(&state.db_health, err, &request),
    };

    // catch up with the latest events
    if let Err(err) = store.load().await {
        return handle_db_error(&state.db_health, err, &request);
    }

    request.extensions_mut().insert(store);

    next.run(request).await
}
