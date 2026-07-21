//! Pre-session pages for the SAML login flow: login start, logout confirmation
//! (TVS T7), and cancelled/failed authentication (T3/L10).

use askama::Template;
use auth_service::{AuthFailure, handle_logout};
use axum::{
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;

use super::{LoggedOutPath, LogoutPath};
use crate::{
    AppState, Context, HtmlTemplate, Locale, LocaleValues, Session, SessionPageValues, filters,
};

#[derive(Template)]
#[template(path = "pg/common/pages/login_start.html")]
struct LoginStartTemplate;

#[derive(Template)]
#[template(path = "pg/common/pages/logged_out.html")]
struct LoggedOutTemplate;

#[derive(Template)]
#[template(path = "pg/common/pages/logout_confirm.html")]
struct LogoutConfirmTemplate;

#[derive(Template)]
#[template(path = "pg/common/pages/auth_error.html")]
struct AuthErrorTemplate;

#[derive(Template)]
#[template(path = "pg/common/pages/auth_cancelled.html")]
struct AuthCancelledTemplate;

#[derive(Template)]
#[template(path = "pg/common/pages/auth_unavailable.html")]
struct AuthUnavailableTemplate;

/// GET `/login`: DigiD start page with login button and flow explanation.
pub async fn login_start(headers: HeaderMap) -> impl IntoResponse {
    HtmlTemplate(
        LoginStartTemplate,
        LocaleValues {
            locale: Locale::from_headers(&headers),
        },
    )
}

/// GET `/logout`: the logout prompt; runs behind the session middleware, which
/// supplies the session and CSRF-checks the prompt's POST.
pub async fn logout(_: LogoutPath, session: Session) -> Response {
    HtmlTemplate(
        LogoutConfirmTemplate,
        SessionPageValues {
            locale: session.locale,
            csrf_token: session.csrf_token().0.clone(),
        },
    )
    .into_response()
}

/// POST `/logout`: starts SP-initiated logout (eID §7.7.1); the session
/// middleware has already verified the CSRF token.
pub async fn logout_submit(
    _: LogoutPath,
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response {
    handle_logout(&state, jar, &LoggedOutPath.to_string()).await
}

/// GET `/logged-out`: post-logout confirmation page (TVS T7, also the SLO
/// landing). Public: by definition there is no session left to show it with.
pub async fn logged_out(_: LoggedOutPath, headers: HeaderMap) -> Response {
    HtmlTemplate(
        LoggedOutTemplate,
        LocaleValues {
            locale: Locale::from_headers(&headers),
        },
    )
    .into_response()
}

/// Response page for a cancelled (T3), failed (L10), or unavailable auth attempt.
pub fn auth_failure_response(failure: AuthFailure, locale: Locale) -> Response {
    let values = LocaleValues { locale };
    match failure {
        AuthFailure::Cancelled => HtmlTemplate(AuthCancelledTemplate, values).into_response(),
        AuthFailure::Error => HtmlTemplate(AuthErrorTemplate, values).into_response(),
        AuthFailure::Unavailable => HtmlTemplate(AuthUnavailableTemplate, values).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    use crate::test_utils::response_body_string;

    #[tokio::test]
    async fn login_start_shows_digid_button_and_explanation() {
        let response = login_start(HeaderMap::new()).await.into_response();
        let body = response_body_string(response).await;
        assert!(body.contains("Inloggen"));
        // The button initiates SSO by POSTing back to /login.
        assert!(body.contains("action=\"/login\""));
        assert!(body.contains("method=\"post\""));
        assert!(body.contains("Kandidaatstelling van de Kiesraad"));
    }

    #[tokio::test]
    async fn logged_out_page_confirms_and_offers_login() {
        let response = logged_out(LoggedOutPath, HeaderMap::new()).await;
        assert!(response.status().is_success());
        let body = response_body_string(response).await;
        assert!(body.contains("U bent uitgelogd"));
        assert!(body.contains("Inloggen"));
    }

    /// Without a session the middleware redirects /logout to the login page.
    #[tokio::test]
    async fn logout_without_session_redirects_to_login() {
        let state = crate::AppState::new_for_tests().await;
        let app = crate::router::create(state.clone()).with_state(state);

        let request = Request::builder()
            .uri("/logout")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/login");
    }

    /// The full logout flow behind the session middleware: the prompt renders
    /// with the session's token, and the CSRF-checked POST removes the session
    /// and lands on the /logged-out confirmation.
    #[tokio::test]
    async fn logout_flow_prompts_then_removes_session() {
        let state = crate::AppState::new_for_tests().await;
        let app = crate::router::create(state.clone()).with_state(state.clone());

        let session = Session::new_test();
        let token = session.token_string();
        let csrf = session.csrf_token().0.clone();
        state.sessions.insert(session).await;
        let cookie = format!("{}={token}", crate::SESSION_COOKIE_NAME);

        let request = Request::builder()
            .uri("/logout")
            .header(header::COOKIE, &cookie)
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&csrf));

        let request = Request::builder()
            .method("POST")
            .uri("/logout")
            .header(header::COOKIE, &cookie)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(format!("csrf_token={csrf}")))
            .unwrap();
        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            &LoggedOutPath.to_string()
        );
        assert!(
            state
                .sessions
                .get_existing(Some(&token))
                .await
                .expect("load session")
                .is_none()
        );
    }

    #[tokio::test]
    async fn error_page_shows_mandated_digid_message() {
        let response = auth_failure_response(AuthFailure::Error, Locale::Nl);
        let body = response_body_string(response).await;
        // TVS L10 requires this literal text on a DigiD result error.
        assert!(body.contains("Inloggen bij deze organisatie is niet gelukt"));
    }

    #[tokio::test]
    async fn cancelled_page_shows_cancellation_notice() {
        let response = auth_failure_response(AuthFailure::Cancelled, Locale::Nl);
        let body = response_body_string(response).await;
        assert!(body.contains("Inloggen geannuleerd"));
    }

    #[tokio::test]
    async fn unavailable_page_shows_temporary_notice() {
        let response = auth_failure_response(AuthFailure::Unavailable, Locale::Nl);
        let body = response_body_string(response).await;
        assert!(body.contains("Inloggen tijdelijk niet mogelijk"));
    }
}
