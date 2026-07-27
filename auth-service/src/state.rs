use crate::{
    config::AuthConfig,
    error::{AuthError, Result},
    keys::{KeyPair, KeySet, load_key_set, load_metadata_tls_cert},
    saml::{
        idp_metadata::{
            IdpMetadata, RdTrust, fetch_and_cache_idp_metadata, load_cached_idp_metadata,
        },
        subject::SubjectId,
    },
};
use axum::{http::HeaderMap, response::Response};
use axum_extra::extract::CookieJar;
use parking_lot::{Mutex, RwLock};
use std::{path::Path, sync::Arc, time::Duration};
use tracing::{debug, error, warn};

/// Upper bound on how often the background task re-fetches the IdP metadata once
/// a descriptor is loaded: the RD metadata is re-fetched at least every 24h, and
/// sooner when its `cacheDuration` (eID §8.5) indicates a shorter window (see
/// [`next_refresh_interval`]). The hard `validUntil` expiry is enforced
/// separately at parse time (`saml::idp_metadata`).
const MAX_METADATA_REFRESH_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// Floor on the computed refresh interval, so a very small (or buggy)
/// `cacheDuration` cannot make the task re-fetch in a tight loop.
const MIN_METADATA_REFRESH_INTERVAL: Duration = Duration::from_secs(15 * 60);

/// How often the background task retries while *no* descriptor is loaded (the RD
/// was unreachable at startup and there was no disk cache). Short enough that the
/// SAML login flow recovers within minutes of the RD coming back rather than
/// waiting a full refresh interval.
const METADATA_RECOVERY_RETRY_INTERVAL: Duration = Duration::from_secs(60);

/// The next refresh delay for a loaded descriptor: its `cacheDuration` clamped to
/// `[MIN_METADATA_REFRESH_INTERVAL, MAX_METADATA_REFRESH_INTERVAL]`, defaulting to
/// the 24h cap when the metadata carries no (parseable) `cacheDuration`.
fn next_refresh_interval(metadata: &IdpMetadata) -> Duration {
    metadata
        .cache_duration
        .unwrap_or(MAX_METADATA_REFRESH_INTERVAL)
        .clamp(MIN_METADATA_REFRESH_INTERVAL, MAX_METADATA_REFRESH_INTERVAL)
}

/// Number of times startup tries to fetch the RD metadata before falling back to
/// the on-disk cache (and, failing that, booting without a descriptor).
const STARTUP_FETCH_ATTEMPTS: u32 = 3;

/// Delay between the startup fetch attempts.
const STARTUP_FETCH_RETRY_DELAY: Duration = Duration::from_secs(2);

/// SAML state shared by the auth-service handlers: configuration, DV/RD keys,
/// and the cached signed SP metadata. Cheap to clone (internally
/// reference-counted), so an embedding application can store one instance on its
/// own state and let the handlers extract it via `FromRef`.
///
/// Replay protection for outgoing AuthnRequest IDs (eID §9.7) is *not* held
/// here: it is delegated to the embedding application via [`AuthState`] so the
/// IDs can be shared across instances (see [`crate::PendingRequests`] for the
/// default in-memory implementation).
#[derive(Clone)]
pub struct AuthServiceState {
    inner: Arc<Inner>,
}

struct Inner {
    auth_config: AuthConfig,
    /// DV keys: signing keys are used to sign outgoing AuthnRequests and metadata;
    /// encryption keys are advertised in metadata and used to decrypt EncryptedID.
    dv_keys: KeySet,
    /// The DV's mTLS client certificate (public key only), advertised as an
    /// extra `use="signing"` KeyDescriptor in the SP metadata per eID §8.3.
    /// `None` when no TLS client certificate is configured or readable; the
    /// mTLS handshake itself reads the cert separately at request time.
    metadata_tls_cert: Option<KeyPair>,
    /// RD descriptor sourced from verified metadata: entity ID, SSO/ARS/SLO
    /// endpoints, signing certs (used to verify ArtifactResponse per §7.6.1
    /// and LogoutResponse per §7.7.2), and encryption certs.
    /// eID §9.2: signature verification keys MUST come from verified metadata.
    /// Wrapped in an `RwLock` so the background refresh task can swap in a
    /// freshly fetched descriptor without disturbing in-flight requests.
    ///
    /// `None` until a descriptor has been loaded: startup may finish without one
    /// (RD unreachable, no disk cache) so the application can still boot, and the
    /// background task fills it in once a fetch succeeds. While it is `None` the
    /// SAML flow is unavailable, which the handlers surface to the user rather
    /// than acting on a missing descriptor.
    rd_metadata: RwLock<Option<Arc<IdpMetadata>>>,
    /// Cached signed SP metadata XML, once built.
    cached_metadata: Mutex<Option<String>>,
}

impl AuthServiceState {
    /// Build the state from the environment ([`AuthConfig::from_env`]). Loads
    /// DV keys from disk and fetches the IdP metadata over HTTP, which yields
    /// the RD entity ID, endpoints, and signing/encryption certs (eID §9.2).
    ///
    /// On a failed fetch the on-disk cache from a previous run is used as a
    /// fallback (see [`load_cached_idp_metadata`]). If there is no cache either,
    /// the state boots without a descriptor: the SAML flow reports itself
    /// unavailable until the background refresh fetches one (see
    /// `load_rd_metadata_at_startup`). Deployments that only exercise the
    /// dev-login bypass construct the state via [`new_empty`](Self::new_empty)
    /// (gated behind `DISABLE_AUTH_SERVICE` in the embedding app) instead of
    /// calling this.
    ///
    /// A background task is spawned that re-fetches the metadata every
    /// `next_refresh_interval`, keeping the cached descriptor fresh within
    /// the RD metadata's `cacheDuration`/`validUntil` (eID §8.5).
    pub async fn new_from_env() -> Result<Self> {
        let auth_config = AuthConfig::from_env()?;
        debug!(
            "[state] AuthConfig loaded: environment={:?}, dv.entity_id={}, rd.metadata_url={}, certs_dir={}",
            auth_config.environment,
            auth_config.dv.entity_id,
            auth_config.rd.metadata_url,
            auth_config.certs_dir.display(),
        );
        let dv_keys = load_key_set(&auth_config.dv.signing, &auth_config.dv.encryption)?;
        debug!(
            "[state] DV keys loaded: signing={}, encryption={}",
            dv_keys.signing.len(),
            dv_keys.encryption.len()
        );
        let rd_metadata = load_rd_metadata_at_startup(
            &auth_config.rd.metadata_url,
            &auth_config.certs_dir,
            RdTrust::for_environment(auth_config.environment),
        )
        .await;
        let state = Self::new(auth_config, dv_keys, rd_metadata);
        state.spawn_metadata_refresh();
        Ok(state)
    }

    pub(crate) fn new(
        auth_config: AuthConfig,
        dv_keys: KeySet,
        rd_metadata: Option<IdpMetadata>,
    ) -> Self {
        let metadata_tls_cert = load_metadata_tls_cert(&auth_config.tls.client_cert);
        Self {
            inner: Arc::new(Inner {
                auth_config,
                dv_keys,
                metadata_tls_cert,
                rd_metadata: RwLock::new(rd_metadata.map(Arc::new)),
                cached_metadata: Mutex::new(None),
            }),
        }
    }

    /// Build a minimal state with default configuration, no keys, and no IdP
    /// metadata. Intended for embedding applications' tests (and dev-login-only
    /// boots) that exercise flows which never perform a live SAML round-trip, so
    /// they need not construct an `AuthConfig`/`KeySet`/`IdpMetadata` themselves.
    /// Any real SAML flow against this state will fail.
    pub fn new_empty() -> Self {
        Self::new(AuthConfig::default(), KeySet::default(), None)
    }

    /// Spawn a background task that keeps the IdP metadata fresh, swapping in
    /// each newly fetched descriptor and refreshing the on-disk cache (eID §8.5).
    /// A failed refresh is logged and the previous descriptor is kept.
    ///
    /// The cadence adapts to whether a descriptor is loaded: once one is present
    /// it refreshes every `next_refresh_interval`; while none is loaded
    /// (the RD was unreachable at startup) it retries every
    /// [`METADATA_RECOVERY_RETRY_INTERVAL`] so login recovers within minutes of
    /// the RD coming back. The task observes the state through a `Weak` handle
    /// and exits once the state is dropped, so it does not keep an otherwise
    /// unused `AuthServiceState` alive.
    fn spawn_metadata_refresh(&self) {
        let weak = Arc::downgrade(&self.inner);

        tokio::spawn(async move {
            loop {
                // Wait first, then refresh: startup already made its attempts, so
                // there is nothing to do on the immediate first pass. The delay is
                // short while we have no descriptor yet; otherwise it is driven by
                // the loaded descriptor's cacheDuration, capped at 24h.
                let delay = match weak.upgrade() {
                    Some(inner) => match inner.rd_metadata.read().as_deref() {
                        Some(metadata) => next_refresh_interval(metadata),
                        None => METADATA_RECOVERY_RETRY_INTERVAL,
                    },
                    None => {
                        debug!("[metadata] Auth state dropped, stopping refresh task");
                        break;
                    }
                };
                tokio::time::sleep(delay).await;

                let Some(inner) = weak.upgrade() else {
                    debug!("[metadata] Auth state dropped, stopping refresh task");
                    break;
                };
                refresh_rd_metadata(
                    &inner.auth_config.rd.metadata_url,
                    &inner.auth_config.certs_dir,
                    &inner.rd_metadata,
                    RdTrust::for_environment(inner.auth_config.environment),
                )
                .await;
            }
        });
    }

    pub(crate) fn auth_config(&self) -> &AuthConfig {
        &self.inner.auth_config
    }

    pub(crate) fn dv_keys(&self) -> &KeySet {
        &self.inner.dv_keys
    }

    /// The DV's mTLS client certificate, published as an additional
    /// `use="signing"` KeyDescriptor in the SP metadata (eID §8.3). `None` when
    /// no TLS client certificate is configured or it could not be read.
    pub(crate) fn metadata_tls_cert(&self) -> Option<&KeyPair> {
        self.inner.metadata_tls_cert.as_ref()
    }

    /// Snapshot of the current RD descriptor, or `None` when none has been
    /// loaded yet (RD unreachable at startup, no disk cache, and no background
    /// refresh has succeeded since). Cheap: clones an `Arc`. The background
    /// refresh task may replace the descriptor between calls, so a caller should
    /// take one handle and use it for a whole request.
    pub(crate) fn rd_metadata(&self) -> Option<Arc<IdpMetadata>> {
        self.inner.rd_metadata.read().clone()
    }

    pub(crate) fn cached_metadata(&self) -> Option<String> {
        self.inner.cached_metadata.lock().clone()
    }

    pub(crate) fn set_cached_metadata(&self, xml: String) {
        *self.inner.cached_metadata.lock() = Some(xml);
    }
}

/// Obtain the RD metadata at startup, returning `None` rather than failing so the
/// application can always boot.
///
/// Tries the live fetch up to [`STARTUP_FETCH_ATTEMPTS`] times (waiting
/// [`STARTUP_FETCH_RETRY_DELAY`] between attempts) to ride out a brief RD outage,
/// then falls back to the on-disk cache from a previous run. If neither yields a
/// descriptor the process still boots with `None`; the SAML login flow then
/// reports itself unavailable (the handlers check for a descriptor) until the
/// background refresh task, which retries quickly while none is loaded, manages
/// to fetch one.
async fn load_rd_metadata_at_startup(
    url: &str,
    certs_dir: &Path,
    trust: RdTrust,
) -> Option<IdpMetadata> {
    for attempt in 1..=STARTUP_FETCH_ATTEMPTS {
        match fetch_and_cache_idp_metadata(url, certs_dir, &trust).await {
            Ok(m) => return Some(m),
            Err(e) => {
                warn!(
                    "[metadata] RD metadata fetch attempt {attempt}/{STARTUP_FETCH_ATTEMPTS} failed: {e}"
                );
                if attempt < STARTUP_FETCH_ATTEMPTS {
                    tokio::time::sleep(STARTUP_FETCH_RETRY_DELAY).await;
                }
            }
        }
    }

    match load_cached_idp_metadata(certs_dir, &trust) {
        Some(cached) => {
            warn!(
                "[metadata] RD metadata fetch exhausted {STARTUP_FETCH_ATTEMPTS} attempts; falling back to on-disk cache"
            );
            Some(cached)
        }
        None => {
            error!(
                "[metadata] No RD metadata after {STARTUP_FETCH_ATTEMPTS} attempts and no disk cache; booting with SAML login unavailable, the background task will keep retrying"
            );
            None
        }
    }
}

/// Re-fetch the RD metadata for the background refresh task, swapping in the new
/// descriptor on success and keeping the previous one (which may be `None`) on
/// failure.
async fn refresh_rd_metadata(
    url: &str,
    certs_dir: &Path,
    slot: &RwLock<Option<Arc<IdpMetadata>>>,
    trust: RdTrust,
) {
    match fetch_and_cache_idp_metadata(url, certs_dir, &trust).await {
        Ok(metadata) => {
            debug!(
                "[metadata] Refreshed RD metadata: entity_id={}, signing_keys={}",
                metadata.entity_id,
                metadata.signing_keys.len()
            );
            *slot.write() = Some(Arc::new(metadata));
        }
        Err(e) => {
            warn!("[metadata] Scheduled RD metadata refresh failed: {e}; keeping previous metadata")
        }
    }
}

/// Application-side flow callbacks the auth-service router needs from the
/// embedding application. SAML state (keys, config) lives in
/// [`AuthServiceState`], which the handlers extract directly via `FromRef`;
/// this trait covers what is application-specific: creating a session after a
/// successful login, tearing one down on logout, and storing the outstanding
/// AuthnRequest IDs used for the `InResponseTo` replay check (eID §9.7). That
/// last item is the application's concern so the IDs can be shared across
/// instances (see [`crate::PendingRequests`] for a ready-made in-memory
/// implementation).
pub trait AuthState: Clone + Send + Sync + 'static {
    /// Called after a SAML Assertion has been fully validated (signature,
    /// InResponseTo, recipient, audience, conditions, LoA) and an acting
    /// SubjectID has been established. The embedding application creates its
    /// own session, sets whatever cookie it uses, and returns the response the
    /// browser should receive, typically a redirect to the post-login landing
    /// page.
    ///
    /// Only the two values the application needs cross this boundary: the
    /// acting `subject_id` (the authenticated identity) and the SAML `name_id`,
    /// which the application persists so it can later build the `LogoutRequest`
    /// (eID §7.7.1). Every validated RD assertion carries a Subject NameID
    /// (§7.6.3, cardinality 1), so `name_id` is always present. The rest of the
    /// validated assertion is internal to the auth-service. An assertion with no
    /// acting SubjectID never reaches here; it is routed to
    /// [`Self::on_authentication_failed`] instead.
    fn on_authenticated(
        &self,
        subject_id: SubjectId,
        name_id: String,
        jar: CookieJar,
        headers: &HeaderMap,
    ) -> impl std::future::Future<Output = Response> + Send;

    /// Called when a SAML flow does not result in a session: the user cancelled
    /// at the IdP ([`AuthFailure::Cancelled`], TVS T3), something went wrong
    /// resolving/validating the response ([`AuthFailure::Error`], TVS L10), or
    /// the auth-service cannot currently run the flow because the RD metadata is
    /// not loaded ([`AuthFailure::Unavailable`]). The auth-service has already
    /// logged the technical detail, so the embedding application is responsible
    /// only for the user-facing page (rendered with its own layout/CSS) and
    /// for tearing down any existing local session (TVS L10). `headers` is
    /// provided for locale negotiation; `jar` lets the application clear its
    /// cookie.
    fn on_authentication_failed(
        &self,
        failure: AuthFailure,
        jar: CookieJar,
        headers: &HeaderMap,
    ) -> impl std::future::Future<Output = Response> + Send;

    /// Terminate the local session for SP-initiated logout. Always returns the
    /// jar with the session cookie cleared, plus the SAML `NameID` if one was
    /// recorded (needed for the `LogoutRequest`, eID §7.7.1).
    fn logout_session(
        &self,
        jar: CookieJar,
    ) -> impl std::future::Future<Output = (CookieJar, Option<String>)> + Send;

    /// Persist an outgoing AuthnRequest ID for the later `InResponseTo` replay
    /// check (eID §7.6.3.5 rule 4 / §9.7). Implementations should sweep entries
    /// older than the retention window (eID §7.5: artifacts are valid for at
    /// most 15 minutes) so abandoned flows cannot accumulate.
    fn register_pending_request(&self, id: String) -> impl std::future::Future<Output = ()> + Send;

    /// Atomically validate and consume an incoming Assertion's `InResponseTo`
    /// against the outstanding AuthnRequest IDs (eID §7.6.3.5 rule 4 / §9.7).
    ///
    /// Returns `true` iff `id` was a still-valid outstanding request: one
    /// registered via [`register_pending_request`](Self::register_pending_request)
    /// and not yet past the retention window. In that case the ID is consumed in
    /// the same step so it can never be matched again, closing the replay window
    /// without a separate check-then-consume round-trip (which would let two
    /// concurrent ACS callbacks for one artifact both succeed). Returns `false`
    /// for an unknown, expired, or already-consumed ID; implementations should
    /// likewise fail closed (return `false`) on a storage error, so the caller
    /// rejects the Assertion rather than admitting an unverified one.
    fn consume_if_pending(&self, id: String) -> impl std::future::Future<Output = bool> + Send;
}

/// Why a SAML authentication attempt ended without an authenticated session.
///
/// Passed to [`AuthState::on_authentication_failed`] so the embedding
/// application can show the appropriate user-facing page. The auth-service logs
/// the technical detail at the failure site; this only conveys *how* to address
/// the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthFailure {
    /// The user cancelled at the IdP (SAML status "AuthnFailed" /
    /// "Authentication cancelled"). TVS "Checklist Testen" v2.1 T3: return the
    /// user to the application with a notice that login was cancelled.
    Cancelled,
    /// Anything else: a DigiD/RD error status (TVS L10) or a protocol/security
    /// validation failure resolving the artifact. The application shows a
    /// generic "login could not be completed" page.
    Error,
    /// The auth-service cannot currently run the SAML flow at all: the RD
    /// metadata has not been loaded (it was unreachable at startup and no
    /// background refresh has succeeded yet), so there is no descriptor to build
    /// or validate messages against. Transient: the application shows a "service
    /// temporarily unavailable, try again shortly" page rather than implying the
    /// user's login itself failed.
    Unavailable,
}

/// Map an internal error to the user-facing failure category, so every handler
/// failure path funnels through [`AuthState::on_authentication_failed`] and the
/// embedding application renders its own error page instead of a bare status
/// code. Transport errors ([`AuthError::Http`]) are transient (`Unavailable`);
/// everything else (signing, templating, bad metadata/config) is `Error`.
impl From<&AuthError> for AuthFailure {
    fn from(e: &AuthError) -> Self {
        match e {
            AuthError::Http(_) => AuthFailure::Unavailable,
            AuthError::Xml(_)
            | AuthError::Crypto(_)
            | AuthError::Config(_)
            | AuthError::Template(_) => AuthFailure::Error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_state() -> AuthServiceState {
        AuthServiceState::new_empty()
    }

    #[test]
    fn cached_metadata_roundtrip() {
        let state = empty_state();
        assert!(state.cached_metadata().is_none());
        state.set_cached_metadata("<xml/>".into());
        assert_eq!(state.cached_metadata().as_deref(), Some("<xml/>"));
    }

    #[test]
    fn empty_state_has_no_rd_metadata() {
        // A state with no loaded descriptor reports the SAML flow as unavailable
        // (the handlers turn this into AuthFailure::Unavailable).
        assert!(empty_state().rd_metadata().is_none());
    }

    fn fixtures_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
    }

    #[test]
    fn accessors_expose_config_keys_and_tls_cert() {
        // Build a state directly from the committed fixtures (no env/network).
        let dir = fixtures_dir();
        let mut cfg = AuthConfig::default().with_certs_dir(dir);
        cfg.dv.entity_id = "urn:test:dv".to_string();
        let keys =
            crate::keys::load_key_set(&cfg.dv.signing, &cfg.dv.encryption).expect("load fixtures");
        let state = AuthServiceState::new(cfg, keys, None);

        assert_eq!(state.auth_config().dv.entity_id, "urn:test:dv");
        assert_eq!(state.dv_keys().signing.len(), 2);
        assert_eq!(state.dv_keys().encryption.len(), 2);
        // `with_certs_dir` points the TLS client cert at the committed fixture,
        // so it is published as an extra signing KeyDescriptor.
        assert!(state.metadata_tls_cert().is_some());
        assert!(state.rd_metadata().is_none());
    }

    #[test]
    fn unreadable_tls_cert_is_omitted_not_fatal() {
        // A configured-but-unreadable TLS client cert is logged and dropped from
        // the metadata rather than failing construction.
        let mut cfg = AuthConfig::default();
        cfg.tls.client_cert = fixtures_dir().join("does-not-exist.pem");
        let state = AuthServiceState::new(cfg, KeySet::default(), None);
        assert!(state.metadata_tls_cert().is_none());
    }

    #[test]
    fn empty_tls_cert_path_is_treated_as_unconfigured() {
        // The default (empty) TLS path means "not configured": no cert, no warning.
        assert!(load_metadata_tls_cert(std::path::Path::new("")).is_none());
    }
}
