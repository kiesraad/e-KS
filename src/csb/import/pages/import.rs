use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::{
    AppError, AppState, AppStoreData, Context, CsbContext, CsbEvent, Form, HtmlTemplate, StreamId,
    filters, political_groups::PoliticalGroup, redirect_success, utils::parse_hash,
};

use crate::csb::import::CsbPoliticalGroups;

use super::CsbImportPath;

#[derive(Template)]
#[template(path = "import/pages/import.html")]
struct CsbImportTemplate {
    csrf_token: String,
    political_groups: Vec<PoliticalGroup>,
}

/// Render the placeholder import page.
pub async fn import(
    _: CsbImportPath,
    context: CsbContext,
    CsbPoliticalGroups(political_groups): CsbPoliticalGroups,
) -> Result<Response, AppError> {
    let csrf_token = context.session.csrf_token.to_string();

    Ok(HtmlTemplate(
        CsbImportTemplate {
            csrf_token,
            political_groups,
        },
        context,
    )
    .into_response())
}

/// Form payload for the import page: the chain hash of the package to import.
#[derive(Debug, Deserialize)]
pub struct ImportForm {
    pub csrf_token: String,
    pub hash: String,
}

/// Import the package identified by the submitted chain hash.
///
/// Locates the political-group event whose hash matches the entry, replays that
/// stream up to the event into an [`AppStoreData`] snapshot (its event log
/// excluded), and records the snapshot in a [`CsbEvent::Import`] persisted under
/// a fresh CSB stream keyed on the source election. The source `stream_id` is
/// carried on the event for reference; it is never reused as the CSB partition,
/// which would collide with the app's own events there.
pub async fn import_submit(
    _: CsbImportPath,
    State(state): State<AppState>,
    context: CsbContext,
    Form(form): Form<ImportForm>,
) -> Result<Response, AppError> {
    context.session.consume_csrf(&form.csrf_token)?;

    let hash_prefix = parse_hash(&form.hash)
        .ok_or_else(|| AppError::UserError("The entered hash is not valid".to_string()))?;

    let (source_stream_id, source_election, event_id) = state
        .store_registry
        .persistence()
        .find_event_by_hash_prefix(&hash_prefix)
        .await?
        .ok_or_else(|| {
            AppError::UserError("No package was found for the entered hash".to_string())
        })?;
    let source_stream_id = StreamId::from(source_stream_id);

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
        let response = import(
            CsbImportPath {},
            CsbContext::new_test(),
            CsbPoliticalGroups(vec![]),
        )
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

        let result = import_submit(
            CsbImportPath {},
            State(state),
            context,
            Form(ImportForm {
                csrf_token,
                hash: "not-a-hash".to_string(),
            }),
        )
        .await;

        assert!(matches!(result, Err(AppError::UserError(_))));
    }

    #[tokio::test]
    async fn import_submit_rejects_unknown_hash() {
        // The in-memory test backend has no event index, so a well-formed hash
        // resolves to no package.
        let state = AppState::new_for_tests().await;
        let context = CsbContext::new_test();
        let csrf_token = context.session.csrf_token.to_string();

        let result = import_submit(
            CsbImportPath {},
            State(state),
            context,
            Form(ImportForm {
                csrf_token,
                hash: "F381 3DE7 96D3 8033".to_string(),
            }),
        )
        .await;

        assert!(matches!(result, Err(AppError::UserError(_))));
    }
}
