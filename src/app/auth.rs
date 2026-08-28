//! The `AuthState` hooks the auth-service invokes on login and logout.

use auth_service::{AuthFailure, AuthState, SubjectId};
use axum::{
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;

use tracing::{error, info, warn};

use crate::{
    AppError, AppState, CsbMainAction, Locale, PgEvent, PgStore, Session, StreamId,
    auth::session_extractor::{
        SESSION_COOKIE_NAME, build_removal_cookie, build_session_cookie, user_agent_hash,
    },
    common::{PgIndexPath, SelectElectionPath, auth_failure_response},
};

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
        let store = self.store_for_stream(stream_id, election, false).await?;
        PgStore::own(store).update(PgEvent::Login).await?;
        session.set_current_election(election);
        Ok(true)
    }

    /// Record `event` on the session's stream, if it has one. Without an election
    /// there is no partition to write to, and the log is the only record.
    async fn record_on_session_stream(&self, session: &Session, event: PgEvent) {
        let (Some(stream_id), Some(election)) = (session.stream_id, session.current_election)
        else {
            return;
        };

        let recorded = match self.store_for_stream(stream_id, election, false).await {
            Ok(store) => PgStore::own(store).update(event).await,
            Err(err) => Err(err),
        };

        if let Err(err) = recorded {
            error!("failed to record session event on stream: {err}");
        }
    }

    /// Record the logout in the audit log of the session's stream: the shared
    /// CSB main stream for committee sessions, the PG stream otherwise.
    async fn record_logout(&self, session: &Session) {
        let Some(user) = session.csb_user.clone() else {
            self.record_on_session_stream(session, PgEvent::Logout)
                .await;
            return;
        };
        let Some(election) = session.current_election else {
            return;
        };

        let recorded = match self.csb_main_store(election).await {
            Ok(store) => store.update(CsbMainAction::Logout.by(user)).await,
            Err(err) => Err(err),
        };

        if let Err(err) = recorded {
            error!("failed to record logout on the CSB main stream: {err}");
        }
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
            Ok(true) => PgIndexPath.to_string(),
            Ok(false) => SelectElectionPath.to_string(),
            Err(err) => return err.into_response(),
        };

        self.sessions.cleanup_expired().await;
        self.sessions.insert(session.clone()).await;

        info!(
            event = "auth.login",
            stream_id = %stream_id,
            scope = session.scope.as_str(),
            "user authenticated"
        );

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
            Some(token) => match self.sessions.remove(&token).await {
                Some(session) => {
                    self.record_logout(&session).await;
                    info!(
                        event = "auth.logout",
                        stream_id = ?session.stream_id,
                        scope = session.scope.as_str(),
                        "user signed out"
                    );
                    Some(session.saml_name_id)
                }
                None => None,
            },
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
        // failed re-authentication never leaves a stale session behind.
        let jar = self.clear_session_cookie(jar).await;
        let locale = Locale::from_headers(headers);

        // No session and no stream, so the log is the only place this can land.
        warn!(
            event = "auth.failed",
            reason = ?failure,
            "authentication failed"
        );

        (jar, auth_failure_response(failure, locale)).into_response()
    }

    async fn register_pending_request(&self, id: String) {
        self.pending_requests.register(id).await;
    }

    async fn consume_if_pending(&self, id: String) -> bool {
        self.pending_requests.consume_if_pending(&id).await
    }
}
