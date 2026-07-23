//! Pre-session pages for the SAML login flow: login start, logout confirmation
//! (TVS T7), and cancelled/failed authentication (T3/L10), plus the
//! [`AuthState`] hooks the auth-service invokes on login and logout.

use askama::Template;
use auth_service::{AuthFailure, AuthState, SubjectId, handle_logout};
use axum::{
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;

use super::{LoggedOutPath, LogoutPath};
use crate::{
    AppError, AppState, Context, HtmlTemplate, Locale, LocaleValues, Session, SessionPageValues,
    StreamId,
    auth::session_extractor::{
        SESSION_COOKIE_NAME, build_removal_cookie, build_session_cookie, user_agent_hash,
    },
    common::{IndexPath, SelectElectionPath},
    filters,
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

impl AppState {
    /// If the stream already has data for some election, prime its store and
    /// set it as the session's current election. Returns `true` when an
    /// election was attached.
    async fn attach_existing_election(
        &self,
        session: &mut Session,
        stream_id: StreamId,
    ) -> Result<bool, AppError> {
        let Some(election) = self
            .existing_elections_for_stream(stream_id)
            .await
            .ok()
            .and_then(|list| list.into_iter().next())
        else {
            return Ok(false);
        };
        self.store_for_stream(stream_id, election, false).await?;
        session.set_current_election(election);
        Ok(true)
    }

    /// Remove the current session (if any) from the store and return a jar with
    /// the session cookie cleared on the client. Used when an authentication
    /// attempt fails so no stale session survives (TVS L10).
    async fn clear_session_cookie(&self, jar: CookieJar) -> CookieJar {
        if let Some(token) = jar.get(SESSION_COOKIE_NAME).map(|c| c.value().to_string()) {
            self.sessions.remove(&token).await;
        }
        jar.remove(build_removal_cookie())
    }
}

impl AuthState for AppState {
    async fn on_authenticated(
        &self,
        subject_id: SubjectId,
        name_id: String,
        jar: CookieJar,
        headers: &HeaderMap,
    ) -> Response {
        // Drop any session the browser already holds, so a pre-login session
        // can't linger server-side (session-fixation defense).
        if let Some(old_token) = jar.get(SESSION_COOKIE_NAME).map(|c| c.value().to_string()) {
            self.sessions.remove(&old_token).await;
        }

        let id_code = subject_id.value;
        let stream_id = self.id_deriver.derive_stream_id(&id_code);

        let mut session = Session::new_with_locale(Locale::from_headers(headers));
        session.set_stream_id(stream_id);
        session.set_user_agent_hash(user_agent_hash(headers));
        session.saml_name_id = name_id;

        let redirect_to = match self.attach_existing_election(&mut session, stream_id).await {
            Ok(true) => IndexPath.to_string(),
            Ok(false) => SelectElectionPath.to_string(),
            Err(err) => return err.into_response(),
        };

        self.sessions.cleanup_expired().await;
        self.sessions.insert(session.clone()).await;

        (
            jar.add(build_session_cookie(&session)),
            Redirect::to(&redirect_to),
        )
            .into_response()
    }

    async fn logout_session(&self, jar: CookieJar) -> (CookieJar, Option<String>) {
        // Drop the session (if any) and always clear the cookie; `Some` means
        // a session was active.
        let name_id = match jar.get(SESSION_COOKIE_NAME).map(|c| c.value().to_string()) {
            Some(token) => self
                .sessions
                .remove(&token)
                .await
                .map(|session| session.saml_name_id),
            None => None,
        };
        (jar.remove(build_removal_cookie()), name_id)
    }

    async fn on_authentication_failed(
        &self,
        failure: AuthFailure,
        jar: CookieJar,
        headers: &HeaderMap,
    ) -> Response {
        // TVS L10: end any existing local session before showing the page, so a
        // failed re-authentication never leaves a stale session behind. The
        // auth-service already logged the technical detail.
        let jar = self.clear_session_cookie(jar).await;
        let locale = Locale::from_headers(headers);
        (jar, auth_failure_response(failure, locale)).into_response()
    }

    async fn register_pending_request(&self, id: String) {
        self.pending_requests.register(id).await;
    }

    async fn consume_if_pending(&self, id: String) -> bool {
        self.pending_requests.consume_if_pending(&id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use axum_extra::extract::cookie::Cookie;
    use secrecy::SecretString;
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

    /// True when the jar emits an expiring `Set-Cookie` for the session cookie.
    fn clears_session_cookie(jar: CookieJar) -> bool {
        let response = (jar, axum::http::StatusCode::OK).into_response();
        response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .any(|cookie| cookie.starts_with(SESSION_COOKIE_NAME) && cookie.contains("Max-Age=0"))
    }

    /// Jar with the session cookie as an *original* request cookie: `remove`
    /// only emits a removal `Set-Cookie` for cookies present on the request.
    fn jar_with_session_cookie(token: &str) -> CookieJar {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            format!("{SESSION_COOKIE_NAME}={token}").parse().unwrap(),
        );
        CookieJar::from_headers(&headers)
    }

    fn subject(value: &str) -> SubjectId {
        SubjectId {
            value: SecretString::from(value.to_string()),
            name_qualifier: "DV".to_string(),
        }
    }

    #[tokio::test]
    async fn on_authenticated_redirects_to_select_election_when_no_data() {
        let state = crate::AppState::new_for_tests().await;

        let response = state
            .on_authenticated(
                subject("999999990"),
                "name-id-xyz".to_string(),
                CookieJar::new(),
                &HeaderMap::new(),
            )
            .await;

        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("redirect location")
            .to_str()
            .unwrap();
        assert_eq!(location, SelectElectionPath.to_string());
    }

    #[cfg(feature = "fixtures")]
    #[tokio::test]
    async fn on_authenticated_attaches_existing_election() {
        let state = crate::AppState::new_for_tests().await;
        let id_code = SecretString::from("999999990");
        let stream_id = state.id_deriver.derive_stream_id(&id_code);

        // Load fixtures so the registry sees events for this (stream, election).
        state
            .store_for_stream(stream_id, crate::ElectionConfig::EK27, true)
            .await
            .unwrap();

        let response = state
            .on_authenticated(
                subject("999999990"),
                "name-id-xyz".to_string(),
                CookieJar::new(),
                &HeaderMap::new(),
            )
            .await;

        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("redirect location")
            .to_str()
            .unwrap();
        assert_eq!(location, IndexPath.to_string());
    }

    #[tokio::test]
    async fn logout_session_without_cookie_has_no_name_id() {
        // Nothing to clear when the request carried no session cookie.
        let state = crate::AppState::new_for_tests().await;
        let (_jar, name_id) = state.logout_session(CookieJar::new()).await;
        assert!(name_id.is_none());
    }

    #[tokio::test]
    async fn logout_session_unknown_session_clears_and_has_no_name_id() {
        let state = crate::AppState::new_for_tests().await;
        let jar = jar_with_session_cookie("no-such-token");
        let (jar, name_id) = state.logout_session(jar).await;
        assert!(name_id.is_none());
        assert!(clears_session_cookie(jar));
    }

    #[tokio::test]
    async fn logout_session_returns_name_id_and_clears_cookie() {
        let state = crate::AppState::new_for_tests().await;
        let mut session = Session::new();
        session.saml_name_id = "name-id-xyz".to_string();
        let token = session.token_string();
        state.sessions.insert(session).await;

        let jar = jar_with_session_cookie(&token);
        let (jar, name_id) = state.logout_session(jar).await;
        assert_eq!(name_id.as_deref(), Some("name-id-xyz"));
        assert!(clears_session_cookie(jar));
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
    async fn logout_session_with_empty_name_id_still_clears_and_ends_session() {
        // An empty NameID must still clear the cookie and end the session.
        let state = crate::AppState::new_for_tests().await;
        let session = Session::new(); // saml_name_id defaults to ""
        let token = session.token_string();
        state.sessions.insert(session).await;

        let jar = jar_with_session_cookie(&token);
        let (jar, name_id) = state.logout_session(jar).await;
        assert_eq!(name_id.as_deref(), Some(""));
        assert!(clears_session_cookie(jar));
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
    async fn on_authenticated_invalidates_prior_session() {
        // A fresh login must not leave the pre-login session alive server-side.
        let state = crate::AppState::new_for_tests().await;
        let old = Session::new();
        let old_token = old.token_string();
        state.sessions.insert(old).await;

        let jar = CookieJar::new().add(Cookie::new(SESSION_COOKIE_NAME, old_token.clone()));
        let _ = state
            .on_authenticated(
                subject("999999990"),
                "name-id-xyz".to_string(),
                jar,
                &HeaderMap::new(),
            )
            .await;

        assert!(
            state
                .sessions
                .get_existing(Some(&old_token))
                .await
                .expect("load session")
                .is_none()
        );
    }

    #[tokio::test]
    async fn on_authentication_failed_terminates_local_session() {
        // TVS L10: a failed authentication must end any existing local session.
        let state = crate::AppState::new_for_tests().await;
        let session = Session::new();
        let token = session.token_string();
        state.sessions.insert(session).await;

        let jar = CookieJar::new().add(Cookie::new(SESSION_COOKIE_NAME, token.clone()));
        let response = state
            .on_authentication_failed(AuthFailure::Error, jar, &HeaderMap::new())
            .await;

        assert!(
            state
                .sessions
                .get_existing(Some(&token))
                .await
                .expect("load session")
                .is_none()
        );
        assert!(response.status().is_success());
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
