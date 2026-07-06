use std::time::Duration;

use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::{
    AppError, AppState, Context, CsbContext, CsbEvent, CsbStoreData, Form, HtmlTemplate, Locale,
    PgStoreData, StreamId, csb::examination::CsbExaminationOverviewPath, filters, redirect_success,
    store::Store, structs::common::BrpClient, trans, utils::parse_hash_prefix,
};

use super::CsbImportPath;

const BRP_COURTESY_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Template)]
#[template(path = "csb/import/pages/import.html")]
struct CsbImportTemplate {
    hash: String,
    error: Option<String>,
}

fn render_import(context: CsbContext, hash: String, error: Option<String>) -> Response {
    HtmlTemplate(CsbImportTemplate { hash, error }, context).into_response()
}

/// Render the placeholder import page.
pub async fn import(_: CsbImportPath, context: CsbContext) -> Result<Response, AppError> {
    Ok(render_import(context, String::new(), None))
}

/// Form payload for the import page: the chain hash of the package to import.
#[derive(Debug, Deserialize)]
pub struct ImportForm {
    pub hash: String,
}

/// Import the package identified by the submitted chain hash.
pub async fn import_submit(
    _: CsbImportPath,
    State(state): State<AppState>,
    context: CsbContext,
    Form(form): Form<ImportForm>,
) -> Result<Response, AppError> {
    let hash = form.hash.clone();
    let locale = context.session.locale;
    match do_import(&state, form, locale).await {
        Ok(response) => Ok(response),
        Err(AppError::UserError(msg)) => Ok(render_import(context, hash, Some(msg))),
        Err(AppError::AmbiguousHash) => Ok(render_import(
            context,
            hash,
            Some(trans!("csb.import.error.ambiguous_hash", locale)),
        )),
        Err(e) => Err(e),
    }
}

/// Locates the political-group event whose hash matches the entry, replays that
/// stream up to the event into an [`PgStoreData`] snapshot (its event log
/// excluded), and records the snapshot in a [`CsbEvent::Import`] persisted under
/// a fresh CSB stream keyed on the source election. The source `stream_id` is
/// carried on the event for reference; it is never reused as the CSB partition,
/// which would collide with the PG stream's own events there.
async fn do_import(
    state: &AppState,
    form: ImportForm,
    locale: Locale,
) -> Result<Response, AppError> {
    let hash_prefix = parse_hash_prefix(&form.hash)
        .ok_or_else(|| AppError::UserError(trans!("csb.import.error.invalid_hash", locale)))?;

    let (source_stream_id, source_election, event_id) = state
        .store_registry
        .persistence()
        .find_event_by_hash_prefix(&hash_prefix)
        .await?
        .ok_or_else(|| AppError::UserError(trans!("csb.import.error.not_found", locale)))?;

    // Reject if this source stream has already been imported.
    for store in state.csb_store_registry.stores_by_scope().await? {
        let already_imported = store.data.read().events.first().is_some_and(|e| {
            matches!(&e.payload, CsbEvent::Import { source_stream_id: sid, .. } if *sid == source_stream_id)
        });
        if already_imported {
            return Err(AppError::UserError(trans!(
                "csb.import.error.already_imported",
                locale
            )));
        }
    }

    // Replay the source stream up to the matched event into a snapshot. Reload
    // first so a cached store that lags the persisted log (e.g. events appended
    // by another instance) still contains the matched event; otherwise
    // `snapshot_until` would silently produce an incomplete snapshot.
    let source_store = state
        .store_for_stream(source_stream_id, source_election, false)
        .await?;
    source_store.load().await?;

    let events = source_store.data.read().events.clone();
    let full_hash = events
        .iter()
        .find(|e| e.event_id == event_id)
        .map(|e| e.hash)
        .ok_or(AppError::GenericNotFound)?;
    let snapshot = PgStoreData::snapshot_until(&events, event_id);

    // Persist the import under a fresh CSB stream.
    let csb_store = state
        .csb_store_for_stream(StreamId::new(), source_election)
        .await?;
    csb_store
        .update(CsbEvent::Import {
            hash: full_hash,
            source_stream_id,
            snapshot: Box::new(snapshot),
        })
        .await?;

    // TODO: get from env at higher level probably
    let brp_client = BrpClient::new("http://localhost:5010", "", "haalcentraal/api/brp/personen");
    do_brp_verification(&csb_store, &brp_client).await?;

    Ok(redirect_success(CsbExaminationOverviewPath {}))
}

pub async fn do_brp_verification(
    store: &Store<CsbStoreData>,
    brp_client: &BrpClient,
) -> Result<(), AppError> {
    let store = store.clone();
    let brp_client = brp_client.clone();

    tokio::task::spawn(async move {
        store.data.write().brp_verification_in_progress = true;

        let mut ticker = tokio::time::interval(BRP_COURTESY_TIMEOUT);
        for person in store.get_persons() {
            ticker.tick().await;

            match brp_client.verify(&person).await {
                Ok(omissions) => {
                    for omission in omissions {
                        if let Err(err) = omission.create(&store).await {
                            tracing::error!(
                                "failed to record BRP omission for {}: {err}",
                                person.id
                            );
                        }
                    }
                }
                Err(err) => {
                    tracing::error!("BRP verification failed for {}: {err}", person.id);
                }
            }
        }

        store.data.write().brp_verification_in_progress = false;
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    use crate::{
        AppState, CsbContext, ElectionConfig, PgEvent, test_utils::response_body_string,
        utils::format_hash,
    };

    /// Populate a political-group stream with a single event in the (in-memory)
    /// test store and return its `(stream_id, formatted chain hash)`.
    async fn seed_source_event(state: &AppState) -> Result<(StreamId, String), AppError> {
        let source_stream = StreamId::new();
        let source_store = state
            .store_for_stream(source_stream, ElectionConfig::EK27, false)
            .await?;
        source_store.update(PgEvent::HideDownloadWarning).await?;

        let hash = source_store.data.read().events[0].hash;
        Ok((source_stream, format_hash(&hash, false)))
    }

    #[tokio::test]
    async fn import_renders_placeholder_page() -> Result<(), AppError> {
        let response = import(CsbImportPath {}, CsbContext::new_test())
            .await
            .unwrap()
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Import"));

        Ok(())
    }

    #[tokio::test]
    async fn import_submit_rejects_unparseable_hash() {
        let state = AppState::new_for_tests().await;
        let context = CsbContext::new_test();

        let response = import_submit(
            CsbImportPath {},
            State(state),
            context,
            Form(ImportForm {
                hash: "not-a-hash".to_string(),
            }),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn import_submit_rejects_unknown_hash() {
        // A fresh in-memory backend has no cached political-group streams, so a
        // well-formed hash resolves to no package.
        let state = AppState::new_for_tests().await;
        let context = CsbContext::new_test();

        let response = import_submit(
            CsbImportPath {},
            State(state),
            context,
            Form(ImportForm {
                hash: "F381 3DE7 96D3 8033".to_string(),
            }),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn import_submit_imports_event_from_in_memory_store() -> Result<(), AppError> {
        let state = AppState::new_for_tests().await;
        let (source_stream, hash) = seed_source_event(&state).await?;

        let context = CsbContext::new_test();

        let response = import_submit(
            CsbImportPath {},
            State(state.clone()),
            context,
            Form(ImportForm { hash }),
        )
        .await?
        .into_response();

        // A successful import redirects to the examination overview.
        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        // The import is recorded under a fresh CSB stream, carrying the source.
        let csb_stores = state.csb_store_registry.stores_by_scope().await?;
        assert_eq!(csb_stores.len(), 1);
        let imported = csb_stores[0].data.read().events.first().is_some_and(|e| {
            matches!(&e.payload, CsbEvent::Import { source_stream_id, .. } if *source_stream_id == source_stream)
        });
        assert!(imported);

        Ok(())
    }

    #[tokio::test]
    async fn import_submit_rejects_already_imported_in_memory_store() -> Result<(), AppError> {
        let state = AppState::new_for_tests().await;
        let (_, hash) = seed_source_event(&state).await?;

        // First import succeeds.
        let context = CsbContext::new_test();
        let response = import_submit(
            CsbImportPath {},
            State(state.clone()),
            context,
            Form(ImportForm { hash: hash.clone() }),
        )
        .await?
        .into_response();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        // Re-importing the same source stream is rejected against the cached CSB
        // store (the in-memory backend persists nothing, so the duplicate check
        // must consult the live registry).
        let context = CsbContext::new_test();
        let response = import_submit(
            CsbImportPath {},
            State(state.clone()),
            context,
            Form(ImportForm { hash }),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        // Only the first import was recorded.
        let csb_stores = state.csb_store_registry.stores_by_scope().await?;
        assert_eq!(csb_stores.len(), 1);

        Ok(())
    }
}
