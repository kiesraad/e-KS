use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::CookieJar;
use secrecy::SecretString;
use serde::Deserialize;

use crate::{
    AppError, AppEvent, AppState, AppStoreData, ElectionConfig, Locale, Session, StreamId,
    auth::session_extractor::build_session_cookie,
    common::{IndexPath, SelectElectionPath},
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
}

pub async fn dev_login(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<DevLoginQuery>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let id_code: SecretString = query
        .bsn
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(SecretString::from)
        .unwrap_or_else(random_bsn);

    let stream_id = state.id_deriver.derive_stream_id(&id_code);
    drop(id_code);

    let locale = request_locale(&headers);
    let mut session = Session::new_with_locale(locale);
    session.set_stream_id(stream_id);

    let redirect_to = if query.select_election.unwrap_or(false) {
        SelectElectionPath.to_string()
    } else {
        let election = ElectionConfig::EK27;
        let load_fixtures = query.fixtures.unwrap_or(false);
        let (store, was_new) = ensure_dev_store(&state, stream_id, load_fixtures, election).await?;

        if was_new {
            store.update(AppEvent::DeveloperLogin { stream_id }).await?;
        }

        session.set_current_election(election);
        IndexPath.to_string()
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
        .get_or_create(stream_id.uuid(), election)
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
