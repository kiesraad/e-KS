//! Session middleware and request extraction.

use axum::{
    extract::{Request, State},
    http::{HeaderMap, Method, header::USER_AGENT},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use chrono::Utc;
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};

use crate::{
    AppError, AppState, Scope, Session, TokenValue,
    app::extension_extractor,
    common::{LoginStartPath, SelectElectionPath},
    csb::examination::CsbExaminationOverviewPath,
    form::generate_csrf_token,
    store::{Store, StoreData},
};

/// Name of the session cookie. The `__Host-` prefix (production only) forbids a
/// `Domain` and requires `Secure` + `Path=/`, blocking sibling-subdomain
/// shadowing. It mandates `Secure`, so dev over http keeps the bare name.
#[cfg(feature = "dev-features")]
pub const SESSION_COOKIE_NAME: &str = "EKS_SESSION_ID";
#[cfg(not(feature = "dev-features"))]
pub const SESSION_COOKIE_NAME: &str = "__Host-EKS_SESSION_ID";

/// CSRF cookie carrying the raw token; `HttpOnly` (server echoes it into forms).
#[cfg(feature = "dev-features")]
pub const CSRF_COOKIE_NAME: &str = "EKS_CSRF";
#[cfg(not(feature = "dev-features"))]
pub const CSRF_COOKIE_NAME: &str = "__Host-EKS_CSRF";

/// Builds the session cookie. Only valid right after creation, while the raw
/// token is still in memory.
pub(crate) fn build_session_cookie(session: &Session) -> Cookie<'static> {
    let token = session
        .reveal_token()
        .expect("build_session_cookie requires a freshly created session with its raw token");
    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, token.to_exposed_string());
    apply_session_cookie_attributes(&mut cookie);
    cookie
}

/// Expired twin of the session cookie for clearing it. Attributes must match the
/// set cookie (esp. `Secure` + `Path=/` for the `__Host-` prefix) or it lingers.
pub(crate) fn build_removal_cookie() -> Cookie<'static> {
    let mut cookie = Cookie::from(SESSION_COOKIE_NAME);
    apply_session_cookie_attributes(&mut cookie);
    cookie
}

/// CSRF cookie with the raw token, same attributes as the session cookie.
pub(crate) fn build_csrf_cookie(raw: TokenValue) -> Cookie<'static> {
    let mut cookie = Cookie::new(CSRF_COOKIE_NAME, raw.0);
    apply_session_cookie_attributes(&mut cookie);
    cookie
}

/// Expired twin of the CSRF cookie, for logout.
pub(crate) fn build_csrf_removal_cookie() -> Cookie<'static> {
    let mut cookie = Cookie::from(CSRF_COOKIE_NAME);
    apply_session_cookie_attributes(&mut cookie);
    cookie
}

fn apply_session_cookie_attributes(cookie: &mut Cookie<'static>) {
    cookie.set_http_only(true);
    #[cfg(feature = "dev-features")]
    cookie.set_secure(false);
    #[cfg(not(feature = "dev-features"))]
    cookie.set_secure(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
}

/// Truncated (64-bit) hex SHA-256 of the request `User-Agent` for session
/// pinning; a missing UA hashes the empty string.
pub(crate) fn user_agent_hash(headers: &HeaderMap) -> String {
    let ua = headers
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    Sha256::digest(ua.as_bytes())
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect()
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

    let mut session = match state.sessions.get_existing(token).await {
        Ok(Some(session)) => session,
        // Send unauthenticated users to the login start page (DigiD button +
        // explanation), not straight into the SAML flow at `/login`.
        Ok(None) => return Redirect::to(&LoginStartPath.to_string()).into_response(),
        // A database error here must not masquerade as "logged out": trip the
        // maintenance gate instead of redirecting to login.
        Err(err) => return crate::handle_db_error(&state.db_health, err, &request),
    };

    if let Some(rejection) =
        reject_on_user_agent_mismatch(&state, &session, request.headers(), token).await
    {
        return rejection;
    }

    // Source the raw CSRF token from its cookie for form rendering; re-mint a
    // missing/stale one only on safe methods, so a mutating request is always
    // verified against the hash that issued its token.
    let is_safe_method = matches!(
        request.method(),
        &Method::GET | &Method::HEAD | &Method::OPTIONS
    );
    let refreshed_csrf = match jar.get(CSRF_COOKIE_NAME).map(|c| c.value().to_string()) {
        Some(value) if session.csrf_matches(&value) => {
            session.set_csrf(TokenValue(value));
            None
        }
        _ if is_safe_method => {
            let raw = generate_csrf_token();
            session.set_csrf(raw.clone());
            Some(raw)
        }
        _ => None,
    };

    session.last_activity = Utc::now();
    state.sessions.insert(session.clone()).await;

    request.extensions_mut().insert(session);

    let response = next.run(request).await;
    match refreshed_csrf {
        Some(raw) => (jar.add(build_csrf_cookie(raw)), response).into_response(),
        None => response,
    }
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
        Err(err) => return crate::handle_db_error(&state.db_health, err, &request),
    };

    // catch up with the latest events
    if let Err(err) = store.load().await {
        return crate::handle_db_error(&state.db_health, err, &request);
    }

    request.extensions_mut().insert(store);

    next.run(request).await
}

// Extracts the current session that the session middleware injected into the
// request extensions.
extension_extractor!(Session, AppError::InternalServerError);
