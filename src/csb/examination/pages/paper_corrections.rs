use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};

use crate::{AppError, AppState, CsbStore, Session, common::IndexPath};

use super::{CsbPaperCorrectionsStartPath, CsbPaperCorrectionsStopPath, CsbPoliticalGroupPath};

/// Put the committee session in paper-corrections mode for this stream. App
/// routes then serve the political group interface over the stream's
/// paper-corrected data (see `store_middleware`).
pub async fn start_paper_corrections(
    _: CsbPaperCorrectionsStartPath,
    State(state): State<AppState>,
    mut session: Session,
    store: CsbStore,
) -> Result<Response, AppError> {
    session.paper_correction_stream_id = Some(store.stream_id);
    // Invalidate forms rendered before the switch, so a stale tab cannot
    // submit changes against this stream's data.
    session.rotate_csrf_token();
    state.sessions.insert(session).await;

    Ok(Redirect::to(&IndexPath.to_string()).into_response())
}

/// Leave paper-corrections mode and return to the stream's examination page.
pub async fn stop_paper_corrections(
    path: CsbPaperCorrectionsStopPath,
    State(state): State<AppState>,
    mut session: Session,
) -> Result<Response, AppError> {
    session.paper_correction_stream_id = None;
    session.rotate_csrf_token();
    state.sessions.insert(session).await;

    Ok(Redirect::to(
        &CsbPoliticalGroupPath {
            stream_id: path.stream_id,
        }
        .to_string(),
    )
    .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    use crate::{AppStoreData, CsbEvent, ElectionConfig, StreamId};

    /// Persist a CSB stream carrying a single import event and return its id.
    async fn seed_csb_store(state: &AppState) -> StreamId {
        let stream_id = StreamId::new();
        let store = state
            .csb_store_for_stream(stream_id, ElectionConfig::EK27)
            .await
            .unwrap();
        store
            .update(CsbEvent::Import {
                hash: [0u8; 32],
                source_stream_id: StreamId::new(),
                snapshot: Box::new(AppStoreData::default()),
            })
            .await
            .unwrap();
        stream_id
    }

    #[tokio::test]
    async fn start_sets_correction_stream_and_redirects_to_app_index() {
        let state = crate::AppState::new_for_tests().await;
        let stream_id = seed_csb_store(&state).await;
        let store = state
            .csb_store_for_stream(stream_id, ElectionConfig::EK27)
            .await
            .unwrap();
        let session = Session::new_test();
        let token = session.token_string();
        let old_csrf = session.csrf_token().to_string();

        let response = start_paper_corrections(
            CsbPaperCorrectionsStartPath { stream_id },
            State(state.clone()),
            session,
            store,
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("Location").unwrap(), "/");
        let stored = state
            .sessions
            .get_existing(Some(token.as_str()))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.paper_correction_stream_id, Some(stream_id));
        // Forms rendered before the switch no longer pass the CSRF guard.
        assert!(!stored.csrf_matches(&old_csrf));
    }

    #[tokio::test]
    async fn stop_clears_correction_stream_and_redirects_to_examination() {
        let state = crate::AppState::new_for_tests().await;
        let stream_id = StreamId::new();
        let mut session = Session::new_test();
        session.paper_correction_stream_id = Some(stream_id);
        let token = session.token_string();
        let old_csrf = session.csrf_token().to_string();

        let response = stop_paper_corrections(
            CsbPaperCorrectionsStopPath { stream_id },
            State(state.clone()),
            session,
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get("Location").unwrap(),
            &format!("/csb/examination/{stream_id}")
        );
        let stored = state
            .sessions
            .get_existing(Some(token.as_str()))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.paper_correction_stream_id, None);
        assert!(!stored.csrf_matches(&old_csrf));
    }
}
