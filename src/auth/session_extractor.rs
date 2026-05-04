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

use crate::{AppError, AppState, Session, common::SelectElectionPath};

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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request as HttpRequest, StatusCode, header},
        middleware,
        routing::get,
    };
    use tower::ServiceExt;

    use crate::{AppState, Session, test_utils::response_body_string};

    /// Redirects to /login when no session cookie is present.
    #[tokio::test]
    async fn middleware_redirects_to_login_without_cookie() {
        let state = AppState::new_for_tests().await;
        let app = Router::new()
            .route(
                "/",
                get(|session: Session| async move { session.token().to_exposed_string() }),
            )
            .layer(middleware::from_fn_with_state(
                state.clone(),
                session_middleware,
            ))
            .with_state(state);

        let response = app
            .oneshot(HttpRequest::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/login");
    }

    /// Reuses the existing session when the cookie is provided.
    #[tokio::test]
    async fn middleware_reuses_session_with_cookie() {
        let state = AppState::new_for_tests().await;
        let app = Router::new()
            .route(
                "/",
                get(|session: Session| async move { session.token().to_exposed_string() }),
            )
            .layer(middleware::from_fn_with_state(
                state.clone(),
                session_middleware,
            ))
            .with_state(state.clone());

        let session = Session::new_test();
        let token = session.token().to_exposed_string();
        state.sessions.insert(session).await;
        let cookie_value = format!("{SESSION_COOKIE_NAME}={token}");

        let response = app
            .oneshot(
                HttpRequest::builder()
                    .uri("/")
                    .header(header::COOKIE, &cookie_value)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        let status = response.status();
        let sets_cookie = response.headers().get(header::SET_COOKIE).is_some();
        let returned_token = response_body_string(response).await;
        assert_eq!(status, StatusCode::OK);
        assert!(!sets_cookie);
        assert_eq!(returned_token, token);
    }
}
