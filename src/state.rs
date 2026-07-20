//! Application state container and request extractors.
//! Holds, among others: configuration, store, and CSRF tokens for handlers.

use auth_service::{AuthFailure, AuthServiceState, AuthState, SubjectId};
use axum::{
    extract::FromRef,
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use secrecy::ExposeSecret;

use crate::{
    AppError, AppStoreData, Config, CsbMainStore, CsbMainStoreData, CsbStore, CsbStoreData,
    DbHealth, ElectionConfig, IdDeriver, PendingRequestStore, Session, SessionStore, StreamId,
    TypstRenderer,
    auth::session_extractor::{
        SESSION_COOKIE_NAME, build_removal_cookie, build_session_cookie, user_agent_hash,
    },
    common::{IndexPath, SelectElectionPath},
    csb::CSB_MAIN_STREAM_ID,
    store::{EventEncryption, Store, StoreRegistry},
};

#[cfg(feature = "fixtures")]
use crate::AppStore;

/// Shared application state for request handlers and extractors.
#[derive(FromRef, Clone)]
pub struct AppState {
    pub config: &'static Config,
    pub store_registry: StoreRegistry<AppStoreData>,
    /// Registry for per-import CSB stores (one per imported political group)
    pub csb_store_registry: StoreRegistry<CsbStoreData>,
    /// Registry for the single global CSB main stream shared by all committee members
    pub csb_main_store_registry: StoreRegistry<CsbMainStoreData>,
    /// Active sessions for this application instance (backed by the configured storage).
    pub sessions: SessionStore,
    /// Outstanding SAML AuthnRequest IDs for the `InResponseTo` replay check
    /// (eID §9.7), backed by the configured storage so they survive restarts
    /// and are shared across instances.
    pub pending_requests: PendingRequestStore,
    pub id_deriver: IdDeriver,
    pub auth_service_state: AuthServiceState,
    pub typst_renderer: TypstRenderer,
    pub db_health: DbHealth,
}

/// Contract the application's request extractors expect from the router
/// state. The supertrait bounds (`Send + Sync`) cover what `AppStore`'s
/// extractor needs; `config()` replaces ad-hoc `FromRef` lookups so each
/// extractor only has to write `S: AppRequestState`.
pub trait AppRequestState: Clone + Send + Sync + 'static {
    fn config(&self) -> &'static Config;
}

impl AppRequestState for AppState {
    fn config(&self) -> &'static Config {
        self.config
    }
}

impl AppState {
    pub async fn new() -> Result<Self, AppError> {
        let config = Config::from_env()?;

        Self::new_with_config(config).await
    }

    pub async fn new_with_config(config: Config) -> Result<Self, AppError> {
        let encryption = EventEncryption::new(&config.encryption_derivation_key);
        let store_registry = StoreRegistry::new(
            config.storage_url.expose_secret().to_string(),
            encryption.clone(),
        )
        .await?;
        // Both CSB registries reuse the app registry's persistence backend
        let csb_store_registry = StoreRegistry::with_persistence(
            store_registry.persistence().clone(),
            encryption.clone(),
        );
        let csb_main_store_registry =
            StoreRegistry::with_persistence(store_registry.persistence().clone(), encryption);
        let sessions = SessionStore::from_storage_url(config.storage_url.expose_secret())?;
        let pending_requests =
            PendingRequestStore::from_storage_url(config.storage_url.expose_secret())?;
        let id_deriver = IdDeriver::new(&config.id_derivation_key);
        let typst_renderer = build_typst_renderer(&config);

        let auth_service_state = if config.disable_auth_service {
            AuthServiceState::new_empty()
        } else {
            AuthServiceState::new_from_env().await?
        };

        Ok(Self {
            config: Box::leak(Box::new(config)),
            store_registry,
            csb_store_registry,
            csb_main_store_registry,
            sessions,
            pending_requests,
            id_deriver,
            auth_service_state,
            typst_renderer,
            db_health: DbHealth::default(),
        })
    }

    pub(crate) async fn store_for_stream(
        &self,
        stream_id: StreamId,
        election: ElectionConfig,
        load_fixtures: bool,
    ) -> Result<Store<AppStoreData>, AppError> {
        #[cfg(feature = "fixtures")]
        {
            self.store_registry
                .get_or_create_with_init(stream_id, election, |store| async move {
                    if store.data.read().events.is_empty() && load_fixtures {
                        crate::fixtures::load(&AppStore::own(store)).await?;
                    }
                    Ok(())
                })
                .await
        }
        #[cfg(not(feature = "fixtures"))]
        {
            let _ = load_fixtures; // avoid unused parameter warning

            self.store_registry.get_or_create(stream_id, election).await
        }
    }

    /// Fetch (or create and load) the CSB store for a (stream, election).
    pub(crate) async fn csb_store_for_stream(
        &self,
        stream_id: StreamId,
        election: ElectionConfig,
    ) -> Result<CsbStore, AppError> {
        self.csb_store_registry
            .get_or_create(stream_id, election)
            .await
    }

    /// Fetch (or create) the global CSB main store for the given election.
    /// All committee members share a single stream [`CSB_MAIN_STREAM_ID`].
    pub(crate) async fn csb_main_store(
        &self,
        election: ElectionConfig,
    ) -> Result<CsbMainStore, AppError> {
        self.csb_main_store_registry
            .get_or_create(CSB_MAIN_STREAM_ID, election)
            .await
    }

    /// List which elections already have persisted data under the user's stream.
    pub(crate) async fn existing_elections_for_stream(
        &self,
        stream_id: StreamId,
    ) -> Result<Vec<ElectionConfig>, AppError> {
        self.store_registry.elections_for_stream(stream_id).await
    }

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

    #[cfg(test)]
    pub async fn new_for_tests() -> Self {
        Self::new_for_tests_with_config(Config::new_test()).await
    }

    /// Test-only constructor that wires up the same dependencies as
    /// [`AppState::new_with_config`] but skips the auth-service env loading
    /// (and IdP metadata fetch) — tests don't run a real IdP.
    #[cfg(test)]
    pub async fn new_for_tests_with_config(config: Config) -> Self {
        let id_deriver = IdDeriver::new(&config.id_derivation_key);
        let encryption = EventEncryption::new(&config.encryption_derivation_key);
        let sessions = SessionStore::from_storage_url(config.storage_url.expose_secret())
            .expect("test SessionStore must initialize");
        let pending_requests =
            PendingRequestStore::from_storage_url(config.storage_url.expose_secret())
                .expect("test PendingRequestStore must initialize");
        let typst_renderer = build_typst_renderer(&config);
        let auth_service_state = AuthServiceState::new_empty();

        let store_registry = StoreRegistry::new(
            config.storage_url.expose_secret().to_string(),
            encryption.clone(),
        )
        .await
        .expect("test StoreRegistry must initialize");
        let csb_store_registry = StoreRegistry::with_persistence(
            store_registry.persistence().clone(),
            encryption.clone(),
        );
        let csb_main_store_registry =
            StoreRegistry::with_persistence(store_registry.persistence().clone(), encryption);

        Self {
            store_registry,
            csb_store_registry,
            csb_main_store_registry,
            config: Box::leak(Box::new(config)),
            sessions,
            pending_requests,
            id_deriver,
            auth_service_state,
            typst_renderer,
            db_health: DbHealth::default(),
        }
    }
}

#[cfg(feature = "embed-typst")]
fn build_typst_renderer(_config: &Config) -> TypstRenderer {
    TypstRenderer::embedded(crate::utils::embed_typst::pdf_context())
}

#[cfg(not(feature = "embed-typst"))]
fn build_typst_renderer(config: &Config) -> TypstRenderer {
    TypstRenderer::http(config.typst_url.clone())
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

        let mut session = Session::new_with_locale(crate::Locale::from_headers(headers));
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
        let locale = crate::Locale::from_headers(headers);
        (jar, crate::common::auth_failure_response(failure, locale)).into_response()
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
    use auth_service::SubjectId;
    use axum::http::header;
    use axum_extra::extract::cookie::Cookie;
    use secrecy::SecretString;

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

    #[tokio::test]
    async fn new_for_tests_sets_config_and_tokens() -> Result<(), AppError> {
        let state = AppState::new_for_tests().await;
        let config = Config::new_test();

        assert_eq!(
            state.config.storage_url.expose_secret(),
            config.storage_url.expose_secret()
        );

        Ok(())
    }

    #[tokio::test]
    async fn existing_elections_for_stream_is_empty_for_fresh_id() {
        let state = AppState::new_for_tests().await;
        let stream_id = crate::StreamId::new();

        let elections = state
            .existing_elections_for_stream(stream_id)
            .await
            .unwrap();
        assert!(elections.is_empty());
    }

    fn subject(value: &str) -> SubjectId {
        SubjectId {
            value: SecretString::from(value.to_string()),
            name_qualifier: "DV".to_string(),
        }
    }

    #[tokio::test]
    async fn on_authenticated_redirects_to_select_election_when_no_data() {
        let state = AppState::new_for_tests().await;

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
        let state = AppState::new_for_tests().await;
        let id_code = SecretString::from("999999990");
        let stream_id = state.id_deriver.derive_stream_id(&id_code);

        // Load fixtures so the registry sees events for this (stream, election).
        state
            .store_for_stream(stream_id, ElectionConfig::EK27, true)
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
        let state = AppState::new_for_tests().await;
        let (_jar, name_id) = state.logout_session(CookieJar::new()).await;
        assert!(name_id.is_none());
    }

    #[tokio::test]
    async fn logout_session_unknown_session_clears_and_has_no_name_id() {
        let state = AppState::new_for_tests().await;
        let jar = jar_with_session_cookie("no-such-token");
        let (jar, name_id) = state.logout_session(jar).await;
        assert!(name_id.is_none());
        assert!(clears_session_cookie(jar));
    }

    #[tokio::test]
    async fn logout_session_returns_name_id_and_clears_cookie() {
        let state = AppState::new_for_tests().await;
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
        let state = AppState::new_for_tests().await;
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
        let state = AppState::new_for_tests().await;
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
        let state = AppState::new_for_tests().await;
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
}
