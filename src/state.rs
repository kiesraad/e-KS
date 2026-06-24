//! Application state container and request extractors.
//! Holds, among others: configuration, store, and CSRF tokens for handlers.

use auth_service::AuthState;
use axum::{
    extract::FromRef,
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use secrecy::ExposeSecret;

use crate::{
    AppError, AppStore, AppStoreData, Config, CsbStore, CsbStoreData, ElectionConfig, IdDeriver,
    Scope, Session, SessionStore, StreamId, TypstRenderer,
    auth::session_extractor::{SESSION_COOKIE_NAME, build_session_cookie},
    common::{IndexPath, SelectElectionPath},
    store::{EventEncryption, StoreRegistry},
    utils::random_bsn,
};

/// Shared application state for request handlers and extractors.
#[derive(FromRef, Clone)]
pub struct AppState {
    pub config: &'static Config,
    pub store_registry: StoreRegistry<AppStoreData>,
    /// Registry for CSB (Centraal Stembureau) stores. Shares the same
    /// persistence backend as `store_registry` but caches a separate
    /// `CsbStoreData` projection.
    pub csb_store_registry: StoreRegistry<CsbStoreData>,
    /// Active sessions for this application instance (backed by the configured storage).
    pub sessions: SessionStore,
    pub id_deriver: IdDeriver,
    pub typst_renderer: TypstRenderer,
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
            Scope::PoliticalGroup,
        )
        .await?;
        // The CSB registry reuses the app registry's (already-initialized)
        // persistence backend; it only differs in the projection it caches and
        // the scope it records on the streams it creates.
        let csb_store_registry = StoreRegistry::with_persistence(
            store_registry.persistence().clone(),
            encryption,
            Scope::CentralElectoralCommittee,
        );
        let sessions = SessionStore::from_storage_url(config.storage_url.expose_secret())?;
        let id_deriver = IdDeriver::new(&config.id_derivation_key);
        let typst_renderer = build_typst_renderer(&config);

        Ok(Self {
            config: Box::leak(Box::new(config)),
            store_registry,
            csb_store_registry,
            sessions,
            id_deriver,
            typst_renderer,
        })
    }

    pub async fn store_for_stream(
        &self,
        stream_id: StreamId,
        election: ElectionConfig,
        load_fixtures: bool,
    ) -> Result<AppStore, AppError> {
        #[cfg(feature = "fixtures")]
        {
            self.store_registry
                .get_or_create_with_init(stream_id.uuid(), election, |store| async move {
                    if store.data.read().events.is_empty() && load_fixtures {
                        crate::fixtures::load(&store).await?;
                    }
                    Ok(())
                })
                .await
        }
        #[cfg(not(feature = "fixtures"))]
        {
            let _ = load_fixtures; // avoid unused parameter warning

            self.store_registry
                .get_or_create(stream_id.uuid(), election)
                .await
        }
    }

    /// Fetch (or create and load) the CSB store for a (stream, election).
    ///
    /// CSB and app stores must never share a `(stream_id, election)` partition:
    /// they write different event types into the same encrypted event stream, so
    /// scope gating (see [`Scope`]) keeps a given session on a single store type.
    pub async fn csb_store_for_stream(
        &self,
        stream_id: StreamId,
        election: ElectionConfig,
    ) -> Result<CsbStore, AppError> {
        self.csb_store_registry
            .get_or_create(stream_id.uuid(), election)
            .await
    }

    /// List which elections already have persisted data under the user's stream.
    pub async fn existing_elections_for_stream(
        &self,
        stream_id: StreamId,
    ) -> Result<Vec<ElectionConfig>, AppError> {
        self.store_registry
            .elections_for_stream(stream_id.uuid())
            .await
    }

    /// Streams the given session is allowed to reach.
    ///
    /// A stream is a `(stream_id, election)` pair, so a session can be entitled
    /// to several streams, including several elections under one `stream_id`.
    ///
    /// - A [`Scope::PoliticalGroup`] session may reach every election under the
    ///   single stream derived from its own identifier.
    /// - A [`Scope::CentralElectoralCommittee`] session may reach every stream
    ///   whose `stream_id` is scoped to the committee.
    pub async fn accessible_streams(
        &self,
        session: &Session,
    ) -> Result<Vec<(uuid::Uuid, ElectionConfig)>, AppError> {
        match session.scope {
            Scope::PoliticalGroup => {
                let Some(stream_id) = session.stream_id else {
                    return Ok(Vec::new());
                };
                let elections = self.existing_elections_for_stream(stream_id).await?;
                Ok(elections
                    .into_iter()
                    .map(|election| (stream_id.uuid(), election))
                    .collect())
            }
            Scope::CentralElectoralCommittee => {
                self.store_registry
                    .streams_by_scope(Scope::CentralElectoralCommittee)
                    .await
            }
        }
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

    #[cfg(test)]
    pub async fn new_for_tests() -> Self {
        let config = Config::new_test();
        let id_deriver = IdDeriver::new(&config.id_derivation_key);
        let encryption = EventEncryption::new(&config.encryption_derivation_key);
        let sessions = SessionStore::from_storage_url(config.storage_url.expose_secret())
            .expect("test SessionStore must initialize");
        let typst_renderer = build_typst_renderer(&config);

        let store_registry = StoreRegistry::new(
            config.storage_url.expose_secret().to_string(),
            encryption.clone(),
            Scope::PoliticalGroup,
        )
        .await
        .expect("test StoreRegistry must initialize");
        let csb_store_registry = StoreRegistry::with_persistence(
            store_registry.persistence().clone(),
            encryption,
            Scope::CentralElectoralCommittee,
        );

        Self {
            store_registry,
            csb_store_registry,
            config: Box::leak(Box::new(config)),
            sessions,
            id_deriver,
            typst_renderer,
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

fn request_locale(headers: &HeaderMap) -> crate::Locale {
    headers
        .get(axum::http::header::ACCEPT_LANGUAGE)
        .and_then(|value| value.to_str().ok())
        .and_then(crate::Locale::from_accept_language)
        .unwrap_or_default()
}

impl AuthState for AppState {
    async fn on_authenticated(&self, jar: CookieJar, headers: &HeaderMap) -> Response {
        let stream_id = {
            let id_code = random_bsn();
            // id_code is intentionally dropped here: after this point the stream
            // is the only user handle we carry, and we never persist the BSN.
            self.id_deriver.derive_stream_id(&id_code)
        };

        let mut session = Session::new_with_locale(request_locale(headers));
        session.set_stream_id(stream_id);

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

    async fn logout_session(&self, jar: CookieJar) -> Option<CookieJar> {
        let token = jar.get(SESSION_COOKIE_NAME)?.value().to_string();
        let _session = self.sessions.remove(&token).await?;
        let mut clear = Cookie::from(SESSION_COOKIE_NAME);
        clear.set_path("/");

        Some(jar.remove(clear))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
