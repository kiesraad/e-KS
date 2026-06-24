//! Public, pre-session pages for the TVS/SAML login flow, all rendered by the
//! eks crate (so they carry the application's own layout and styling) and all
//! reachable without a session — they are mounted outside the session
//! middleware:
//!
//! - the login *start* page (DigiD button + a short explanation), shown instead
//!   of redirecting the browser straight to TVS;
//! - the logout confirmation page (TVS "Checklist Testen" v2.1 T7);
//! - the cancelled / failed authentication pages (T3 / L10), reached when the
//!   auth-service calls
//!   [`AuthState::on_authentication_failed`](auth_service::AuthState::on_authentication_failed).

use askama::Template;
use auth_service::{AuthFailure, AuthServiceState, handle_logout};
use axum::{
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;

use crate::{AppState, Context, HtmlTemplate, Locale, SESSION_COOKIE_NAME, filters};

#[derive(Template)]
#[template(path = "common/pages/login_start.html")]
struct LoginStartTemplate;

#[derive(Template)]
#[template(path = "common/pages/logged_out.html")]
struct LoggedOutTemplate;

#[derive(Template)]
#[template(path = "common/pages/auth_error.html")]
struct AuthErrorTemplate;

#[derive(Template)]
#[template(path = "common/pages/auth_cancelled.html")]
struct AuthCancelledTemplate;

/// Minimal template values for a standalone, pre-session page: only the locale
/// is known, which is all the `trans` filter needs. The full request
/// [`Context`] is unavailable here because there is no session or election yet.
struct PublicPageValues {
    locale: Locale,
}

impl askama::Values for PublicPageValues {
    fn get_value<'a>(&'a self, key: &str) -> Option<&'a dyn std::any::Any> {
        match key {
            "locale" => Some(&self.locale as &dyn std::any::Any),
            _ => None,
        }
    }
}

/// Resolve the display locale from the request's `Accept-Language`, falling back
/// to the application default. These pages run before a session exists, so the
/// session locale is not yet available.
fn request_locale(headers: &HeaderMap) -> Locale {
    headers
        .get(axum::http::header::ACCEPT_LANGUAGE)
        .and_then(|value| value.to_str().ok())
        .and_then(Locale::from_accept_language)
        .unwrap_or_default()
}

/// GET `/login` — DigiD start page. Shows the DigiD login button (which POSTs
/// back to `/login` to begin SAML SSO) and a short explanation of the flow,
/// instead of redirecting the browser straight to TVS.
pub async fn login_start(headers: HeaderMap) -> impl IntoResponse {
    HtmlTemplate(
        LoginStartTemplate,
        PublicPageValues {
            locale: request_locale(&headers),
        },
    )
}

/// GET `/logout` — the whole sign-out flow under a single path.
///
/// With an active session it starts SP-initiated logout (eID §7.7.1) by
/// delegating to the auth-service `handle_logout`, which auto-POSTs a signed
/// LogoutRequest to TVS. With no session it renders the post-logout
/// confirmation page (TVS T7) — which is also where the SLO round-trip lands
/// (`post_logout_redirect` points back here), since the session has been
/// cleared by then.
pub async fn logout(
    State(state): State<AppState>,
    State(auth_state): State<AuthServiceState>,
    jar: CookieJar,
    headers: HeaderMap,
) -> Response {
    let token = jar.get(SESSION_COOKIE_NAME).map(|c| c.value().to_string());
    if state
        .sessions
        .get_existing(token.as_deref())
        .await
        .is_some()
    {
        handle_logout(State(state), State(auth_state), jar).await
    } else {
        logged_out_page(&headers)
    }
}

/// Render the post-logout confirmation page (TVS T7), confirming the user is
/// logged out and offering to log in again.
fn logged_out_page(headers: &HeaderMap) -> Response {
    HtmlTemplate(
        LoggedOutTemplate,
        PublicPageValues {
            locale: request_locale(headers),
        },
    )
    .into_response()
}

/// Render the page for a cancelled (T3) or failed (L10) authentication attempt,
/// localised via the request's `Accept-Language`.
pub fn auth_failure_response(failure: AuthFailure, locale: Locale) -> Response {
    let values = PublicPageValues { locale };
    match failure {
        AuthFailure::Cancelled => HtmlTemplate(AuthCancelledTemplate, values).into_response(),
        AuthFailure::Error => HtmlTemplate(AuthErrorTemplate, values).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_utils::response_body_string;

    #[tokio::test]
    async fn login_start_shows_digid_button_and_explanation() {
        let response = login_start(HeaderMap::new()).await.into_response();
        let body = response_body_string(response).await;
        assert!(body.contains("Inloggen met DigiD"));
        // The button initiates SSO by POSTing back to /login.
        assert!(body.contains("action=\"/login\""));
        assert!(body.contains("method=\"post\""));
        assert!(body.contains("Kandidaatstelling van de Kiesraad"));
    }

    #[tokio::test]
    async fn logged_out_page_confirms_and_offers_login() {
        let response = logged_out_page(&HeaderMap::new());
        let body = response_body_string(response).await;
        assert!(body.contains("U bent uitgelogd"));
        assert!(body.contains("Inloggen met DigiD"));
    }

    #[tokio::test]
    async fn logout_without_session_renders_confirmation() {
        let state = crate::AppState::new_for_tests().await;
        let response = logout(
            axum::extract::State(state.clone()),
            axum::extract::State(state.auth_service_state.clone()),
            CookieJar::new(),
            HeaderMap::new(),
        )
        .await;
        assert!(response.status().is_success());
        let body = response_body_string(response).await;
        assert!(body.contains("U bent uitgelogd"));
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
}
