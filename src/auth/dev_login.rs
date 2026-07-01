use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::CookieJar;
use secrecy::SecretString;
use serde::Deserialize;

use crate::{
    AppError, AppEvent, AppState, AppStoreData, CsbMainEvent, ElectionConfig, Locale, Scope,
    Session, StreamId,
    auth::session_extractor::build_session_cookie,
    common::{IndexPath, SelectElectionPath},
    csb::examination::CsbExaminationOverviewPath,
    political_groups::PoliticalGroup,
    store::Store,
    utils::random_bsn,
};

pub const DEV_LOGIN_PATH: &str = "/dev/login";

#[derive(Debug, Deserialize)]
pub struct DevLoginQuery {
    bsn: Option<String>,
    fixtures: Option<bool>,
    select_election: Option<bool>,
    csb: Option<bool>,
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
) -> Result<impl IntoResponse, AppError> {
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

    let locale = request_locale(&headers);
    let mut session = Session::new_with_locale(locale);
    session.set_stream_id(stream_id);
    session.set_scope(scope);

    let load_fixtures = query.fixtures.unwrap_or(false);

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

            CsbExaminationOverviewPath {}.to_string()
        }
        Scope::ImportedByCsb => return Err(AppError::Unauthorised),
        Scope::PoliticalGroup => {
            if query.select_election.unwrap_or(false) {
                SelectElectionPath.to_string()
            } else {
                let election = ElectionConfig::EK27;
                let (store, was_new) =
                    ensure_dev_store(&state, stream_id, load_fixtures, election).await?;

                if was_new {
                    store.update(AppEvent::DeveloperLogin { stream_id }).await?;
                }

                session.set_current_election(election);
                IndexPath.to_string()
            }
        }
    };

    state.sessions.cleanup_expired().await;
    state.sessions.insert(session.clone()).await;

    Ok((
        jar.add(build_session_cookie(&session)),
        Redirect::to(&redirect_to),
    ))
}

pub(crate) fn request_locale(headers: &axum::http::HeaderMap) -> Locale {
    headers
        .get(axum::http::header::ACCEPT_LANGUAGE)
        .and_then(|value| value.to_str().ok())
        .and_then(Locale::from_accept_language)
        .unwrap_or_default()
}

async fn ensure_dev_store(
    state: &AppState,
    stream_id: StreamId,
    load_fixtures: bool,
    election: ElectionConfig,
) -> Result<(Store<AppStoreData>, bool), AppError> {
    let store = state
        .store_registry
        .get_or_create(stream_id, election)
        .await?;
    let store_is_empty = store.data.read().events.is_empty();

    if store_is_empty {
        PoliticalGroup::default().create(&store).await?;
    }

    if load_fixtures {
        #[cfg(feature = "fixtures")]
        {
            crate::fixtures::load(&store).await?;
            return Ok((store, store_is_empty));
        }
    }

    Ok((store, store_is_empty))
}
