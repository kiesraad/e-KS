use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::{
    AppError, AppState, AppStoreData, Context, CsbContext, CsbEvent, Form, HtmlTemplate, Locale,
    Scope, StreamId, filters, redirect_success, trans, utils::parse_hash_prefix,
};

use super::CsbImportPath;

#[derive(Template)]
#[template(path = "import/pages/import.html")]
struct CsbImportTemplate {
    csrf_token: String,
    hash: String,
    error: Option<String>,
}

fn render_import(context: CsbContext, hash: String, error: Option<String>) -> Response {
    HtmlTemplate(
        CsbImportTemplate {
            csrf_token: context.session.csrf_token.to_string(),
            hash,
            error,
        },
        context,
    )
    .into_response()
}

/// Render the placeholder import page.
pub async fn import(_: CsbImportPath, context: CsbContext) -> Result<Response, AppError> {
    Ok(render_import(context, String::new(), None))
}

/// Form payload for the import page: the chain hash of the package to import.
#[derive(Debug, Deserialize)]
pub struct ImportForm {
    pub csrf_token: String,
    pub hash: String,
}

/// Import the package identified by the submitted chain hash.
pub async fn import_submit(
    _: CsbImportPath,
    State(state): State<AppState>,
    context: CsbContext,
    Form(form): Form<ImportForm>,
) -> Result<Response, AppError> {
    context.session.consume_csrf(&form.csrf_token)?;

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
/// stream up to the event into an [`AppStoreData`] snapshot (its event log
/// excluded), and records the snapshot in a [`CsbEvent::Import`] persisted under
/// a fresh CSB stream keyed on the source election. The source `stream_id` is
/// carried on the event for reference; it is never reused as the CSB partition,
/// which would collide with the app's own events there.
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
    let source_stream_id = StreamId::from(source_stream_id);

    // Reject if this source stream has already been imported.
    for (stream_id, election) in state
        .csb_store_registry
        .streams_by_scope(Scope::CentralElectoralCommittee)
        .await?
    {
        let store = state
            .csb_store_registry
            .get_or_create(stream_id, election)
            .await?;
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

    // Replay the source stream up to the matched event into a snapshot.
    let source_store = state
        .store_for_stream(source_stream_id, source_election, false)
        .await?;
    let snapshot = AppStoreData::snapshot_until(&source_store.get_events(), event_id);

    // Persist the import under a fresh CSB stream.
    let csb_store = state
        .csb_store_for_stream(StreamId::new(), source_election)
        .await?;
    csb_store
        .update(CsbEvent::Import {
            hash: form.hash,
            source_stream_id,
            snapshot: Box::new(snapshot),
        })
        .await?;

    Ok(redirect_success(CsbImportPath {}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    use crate::{AppState, CsbContext, test_utils::response_body_string};

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
    async fn import_submit_rejects_invalid_csrf() {
        let state = AppState::new_for_tests().await;

        let result = import_submit(
            CsbImportPath {},
            State(state),
            CsbContext::new_test(),
            Form(ImportForm {
                csrf_token: "wrong".to_string(),
                hash: "F381 3DE7".to_string(),
            }),
        )
        .await;

        assert!(matches!(result, Err(AppError::CsrfTokenInvalid)));
    }

    #[tokio::test]
    async fn import_submit_rejects_unparseable_hash() {
        let state = AppState::new_for_tests().await;
        let context = CsbContext::new_test();
        let csrf_token = context.session.csrf_token.to_string();

        let response = import_submit(
            CsbImportPath {},
            State(state),
            context,
            Form(ImportForm {
                csrf_token,
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
        // The in-memory test backend has no event index, so a well-formed hash
        // resolves to no package.
        let state = AppState::new_for_tests().await;
        let context = CsbContext::new_test();
        let csrf_token = context.session.csrf_token.to_string();

        let response = import_submit(
            CsbImportPath {},
            State(state),
            context,
            Form(ImportForm {
                csrf_token,
                hash: "F381 3DE7 96D3 8033".to_string(),
            }),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
