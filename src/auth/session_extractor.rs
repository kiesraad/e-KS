//! Session middleware and request extraction.

use std::time::Instant;

use axum::{
    extract::{FromRequestParts, Request, State},
    http::request::Parts,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};

use crate::{AppError, AppState, Locale, Session};

/// Name of the session cookie used by the application.
pub const SESSION_COOKIE_NAME: &str = "EKS_SESSION_ID";

/// Builds a secure, HTTP-only cookie that carries the session token.
fn build_session_cookie(session: &Session) -> Cookie<'static> {
    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, session.token().to_exposed_string());
    cookie.set_http_only(true);
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
    let token = jar.get(SESSION_COOKIE_NAME).map(|cookie| cookie.value());

    let mut session = match state.sessions.get_existing(token) {
        Some(existing) => existing,
        None => {
            // TEMPORARY (pre-auth): reuse any existing session when the cookie is missing.
            // This must be removed once login/auth flows own session creation.
            if token.is_none()
                && let Some(mut existing) = state.sessions.get_any_active_for_dev()
            {
                existing.last_activity = Instant::now();
                state.sessions.insert(existing.clone());
                request.extensions_mut().insert(existing.clone());
                let response = next.run(request).await;
                let jar = jar.add(build_session_cookie(&existing));

                return (jar, response).into_response();
            }

            // TODO: only create a new session after a successfull login
            state.sessions.cleanup_expired();
            let locale = request
                .headers()
                .get(axum::http::header::ACCEPT_LANGUAGE)
                .and_then(|value| value.to_str().ok())
                .and_then(Locale::from_accept_language)
                .unwrap_or_default();
            let mut new_session = Session::new_with_locale(locale);
            new_session.set_political_group(uuid::Uuid::nil().into());
            state.sessions.insert(new_session.clone());
            request.extensions_mut().insert(new_session.clone());
            let response = next.run(request).await;
            let jar = jar.add(build_session_cookie(&new_session));

            return (jar, response).into_response();
        }
    };

    session.last_activity = Instant::now();
    state.sessions.insert(session.clone());
    request.extensions_mut().insert(session.clone());

    next.run(request).await
}

/// Middleware that resolves the scoped store for the session's political group.
pub async fn store_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(political_group_id) = request
        .extensions()
        .get::<Session>()
        .and_then(|session| session.political_group_id)
    else {
        return next.run(request).await;
    };

    let store = match state.store_for_political_group(political_group_id).await {
        Ok(store) => store,
        Err(err) => return err.into_response(),
    };

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

    /// Returns the session token and sets a cookie on first request.
    #[tokio::test]
    async fn middleware_sets_cookie_and_injects_session() {
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

        assert_eq!(response.status(), StatusCode::OK);
        let set_cookie = response
            .headers()
            .get(header::SET_COOKIE)
            .and_then(|value| value.to_str().ok())
            .expect("set-cookie header");
        assert!(set_cookie.contains(SESSION_COOKIE_NAME));
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
            .with_state(state);

        let first = app
            .clone()
            .oneshot(HttpRequest::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .expect("response");

        let set_cookie = first
            .headers()
            .get(header::SET_COOKIE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string)
            .expect("set-cookie header");
        let first_token = response_body_string(first).await;
        let cookie_value = set_cookie.split(';').next().expect("cookie value");

        let second = app
            .oneshot(
                HttpRequest::builder()
                    .uri("/")
                    .header(header::COOKIE, cookie_value)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        let second_status = second.status();
        let second_sets_cookie = second.headers().get(header::SET_COOKIE).is_some();
        let second_token = response_body_string(second).await;
        assert_eq!(second_status, StatusCode::OK);
        assert!(!second_sets_cookie);
        assert_eq!(first_token, second_token);
    }
}
