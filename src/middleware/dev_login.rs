use axum::{
    extract::{Query, State},
    http::{HeaderName, HeaderValue},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use secrecy::SecretString;
use serde::Deserialize;

use crate::{
    AppError, AppState, CsbMainEvent, ElectionConfig, Locale, PgEvent, PgStoreData, Scope, Session,
    StreamId,
    auth::session_extractor::{build_session_cookie, user_agent_hash},
    common::{DisplayName, IndexPath, SelectElectionPath},
    csb::index::CsbIndexPath,
    political_groups::PoliticalGroup,
    store::Store,
    utils::{format_hash, random_bsn},
};

pub const DEV_LOGIN_PATH: &str = "/dev/login";

/// Response header on the dev-login redirect carrying the chain hash of the
/// stream's last event, so end-to-end tests can drive the CSB import flow.
pub const LAST_EVENT_HASH_HEADER: HeaderName = HeaderName::from_static("x-last-event-hash");

/// Placeholder `NameID` for dev-login sessions, which skip the SAML flow.
const DEV_LOGIN_NAME_ID: &str = "dev-login-placeholder-name-id";

#[derive(Debug, Deserialize)]
pub struct DevLoginQuery {
    bsn: Option<String>,
    fixtures: Option<bool>,
    select_election: Option<bool>,
    csb: Option<bool>,
    name: Option<String>,
}

/// Dev login. By default the session and its stream are scoped to
/// [`Scope::PoliticalGroup`] and the group only sees its own stream.
///
/// When `csb=true` the login is instead a member of the central electoral
/// committee (CSB): the session is scoped to
/// [`Scope::CentralElectoralCommittee`], giving access to the shared committee
/// main stream and to all streams scoped to [`Scope::ImportedByCsb`]. No
/// per-session committee stream is created.
pub async fn dev_login(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<DevLoginQuery>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let scope = if query.csb.unwrap_or(false) {
        Scope::CentralElectoralCommittee
    } else {
        Scope::PoliticalGroup
    };
    perform_dev_login(state, jar, query, headers, scope).await
}

async fn perform_dev_login(
    state: AppState,
    jar: CookieJar,
    query: DevLoginQuery,
    headers: axum::http::HeaderMap,
    scope: Scope,
) -> Result<Response, AppError> {
    let fixture_name: Option<DisplayName> = query
        .name
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(str::parse)
        .transpose()
        .map_err(|_| AppError::UserError("invalid political group name".to_string()))?;

    let id_code: SecretString = query
        .bsn
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(SecretString::from)
        .unwrap_or_else(random_bsn);

    // The stream id is derived from the BSN only, not the scope. In real use a
    // person has a single role, so a BSN maps to one scope. These dev endpoints
    // don't enforce that: logging in with the *same* BSN as both a political
    // group and the committee would point both an app store and a CSB store at
    // the same `(stream_id, election)` partition (one cipher, two event types),
    // which fails to decode. Accepted as a dev-only limitation; use distinct
    // BSNs for the two roles.
    let stream_id = state.id_deriver.derive_stream_id(&id_code);
    drop(id_code);

    let locale = Locale::from_headers(&headers);
    let mut session = Session::new_with_locale(locale);
    session.set_stream_id(stream_id);
    session.set_scope(scope);
    session.set_user_agent_hash(user_agent_hash(&headers));
    session.saml_name_id = DEV_LOGIN_NAME_ID.to_string();

    let load_fixtures = query.fixtures.unwrap_or(false);
    let mut last_event_hash = None;

    let redirect_to = match scope {
        Scope::CentralElectoralCommittee => {
            // All committee members share a single stream
            // The session's own stream_id is not used for CSB state (other than for logging)
            let election = ElectionConfig::EK27;
            let store = state.csb_main_store(election).await?;
            store
                .update(CsbMainEvent::DeveloperLogin { stream_id })
                .await?;
            session.set_current_election(election);

            #[cfg(feature = "fixtures")]
            if load_fixtures {
                crate::csb::import::fixture::import_csb_fixture(&state, election).await?;
            }

            CsbIndexPath {}.to_string()
        }
        Scope::ImportedByCsb => return Err(AppError::Unauthorised),
        Scope::PoliticalGroup => {
            if query.select_election.unwrap_or(false) {
                SelectElectionPath.to_string()
            } else {
                let election = ElectionConfig::EK27;
                let (store, was_new) =
                    ensure_dev_store(&state, stream_id, load_fixtures, fixture_name, election)
                        .await?;

                if was_new {
                    store.update(PgEvent::DeveloperLogin { stream_id }).await?;
                }
                last_event_hash = store
                    .data
                    .read()
                    .events
                    .last()
                    .map(|e| format_hash(&e.hash, false));

                session.set_current_election(election);
                IndexPath.to_string()
            }
        }
    };

    state.sessions.cleanup_expired().await;
    state.sessions.insert(session.clone()).await;

    let mut response = (
        jar.add(build_session_cookie(&session)),
        Redirect::to(&redirect_to),
    )
        .into_response();

    if let Some(hash) = last_event_hash {
        // The hash is uppercase hex with spaces, always a valid header value
        let value = HeaderValue::from_str(&hash).map_err(|_| AppError::InternalServerError)?;
        response.headers_mut().insert(LAST_EVENT_HASH_HEADER, value);
    }

    Ok(response)
}

async fn ensure_dev_store(
    state: &AppState,
    stream_id: StreamId,
    load_fixtures: bool,
    fixture_name: Option<DisplayName>,
    election: ElectionConfig,
) -> Result<(Store<PgStoreData>, bool), AppError> {
    let store = state
        .store_registry
        .get_or_create(stream_id, election)
        .await?;
    let store_is_empty = store.data.read().events.is_empty();

    if store_is_empty {
        PoliticalGroup::default()
            .create(&crate::PgStore::own(store.clone()))
            .await?;
    }

    if load_fixtures {
        #[cfg(feature = "fixtures")]
        {
            crate::fixtures::load(&crate::PgStore::own(store.clone()), fixture_name).await?;
            return Ok((store, store_is_empty));
        }
    }
    #[cfg(not(feature = "fixtures"))]
    let _ = fixture_name;

    Ok((store, store_is_empty))
}
