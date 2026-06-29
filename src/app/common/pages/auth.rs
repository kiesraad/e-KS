//! Pre-session pages for the SAML login flow: login start, logout confirmation
//! (TVS T7), and cancelled/failed authentication (T3/L10).

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

#[derive(Template)]
#[template(path = "common/pages/auth_unavailable.html")]
struct AuthUnavailableTemplate;

/// Template values for pre-session pages: only locale is known (no session/election yet).
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

/// Resolve locale from `Accept-Language`, falling back to the application default.
fn request_locale(headers: &HeaderMap) -> Locale {
    headers
        .get(axum::http::header::ACCEPT_LANGUAGE)
        .and_then(|value| value.to_str().ok())
        .and_then(Locale::from_accept_language)
        .unwrap_or_default()
}

/// GET `/login`: DigiD start page with login button and flow explanation.
pub async fn login_start(headers: HeaderMap) -> impl IntoResponse {
    HtmlTemplate(
        LoginStartTemplate,
        PublicPageValues {
            locale: request_locale(&headers),
        },
    )
}

/// GET `/logout`: starts SP-initiated logout (eID §7.7.1) when a session is
/// active, or renders the post-logout confirmation (TVS T7) when there isn't one.
/// The SLO round-trip lands back here after the session is cleared.
pub async fn logout(
    State(state): State<AppState>,
    State(auth_state): State<AuthServiceState>,
    jar: CookieJar,
    headers: HeaderMap,
) -> Response {
    let token = jar.get(SESSION_COOKIE_NAME).map(|c| c.value().to_string());
    // On a storage error we can't confirm a session, so fall through to the
    // post-logout page rather than failing the logout.
    if matches!(
        state.sessions.get_existing(token.as_deref()).await,
        Ok(Some(_))
    ) {
        handle_logout(State(state), State(auth_state), jar).await
    } else {
        logged_out_page(&headers)
    }
}

/// Post-logout confirmation page (TVS T7).
fn logged_out_page(headers: &HeaderMap) -> Response {
    HtmlTemplate(
        LoggedOutTemplate,
        PublicPageValues {
            locale: request_locale(headers),
        },
    )
    .into_response()
}

/// Response page for a cancelled (T3), failed (L10), or unavailable auth attempt.
pub fn auth_failure_response(failure: AuthFailure, locale: Locale) -> Response {
    let values = PublicPageValues { locale };
    match failure {
        AuthFailure::Cancelled => HtmlTemplate(AuthCancelledTemplate, values).into_response(),
        AuthFailure::Error => HtmlTemplate(AuthErrorTemplate, values).into_response(),
        AuthFailure::Unavailable => HtmlTemplate(AuthUnavailableTemplate, values).into_response(),
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
        assert!(body.contains("Inloggen"));
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
        assert!(body.contains("Inloggen"));
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

    #[tokio::test]
    async fn unavailable_page_shows_temporary_notice() {
        let response = auth_failure_response(AuthFailure::Unavailable, Locale::Nl);
        let body = response_body_string(response).await;
        assert!(body.contains("Inloggen tijdelijk niet mogelijk"));
    }
}
