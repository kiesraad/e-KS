use askama::Template;
use axum::response::{IntoResponse, Redirect, Response};
use rand::{RngExt, rng};

use crate::{
    AppError, Context, CsbContext,
    CsbEvent::{self},
    CsbStore, ElectionConfig, HtmlTemplate,
    csb::examination::{
        extractors::CsbPoliticalGroup,
        pages::{CsbPoliticalGroupPath, CsbPoliticalGroupToggleFinishPath},
        structs::CsbCandidateList,
    },
    filters,
};

#[derive(Template)]
#[template(path = "examination/pages/political_group.html")]
struct CsbPoliticalGroupTemplate {
    // TODO make election part of CsbContext?
    election: ElectionConfig,
    political_group: CsbPoliticalGroup,
    all_brp_error_count: usize,
    candidate_lists: Vec<CsbCandidateList>,
    general_brp_error_count: usize,
    restoration_count: usize,
}

/// Render the placeholder political group overview page.
pub async fn overview(
    _: CsbPoliticalGroupPath,
    context: CsbContext,
    store: CsbStore,
) -> Result<Response, AppError> {
    let store_data = &store.data.read();
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    let general_brp_error_count = rng().random_range(0..=2);
    let candidate_lists = store_data
        .imported_data
        .candidate_lists
        .values()
        .cloned()
        .map(CsbCandidateList::placeholder)
        .collect::<Vec<_>>();
    let all_brp_error_count = candidate_lists
        .iter()
        .map(|cl| cl.brp_error_count)
        .sum::<usize>()
        + general_brp_error_count;
    Ok(HtmlTemplate(
        CsbPoliticalGroupTemplate {
            election: store.election,
            political_group,
            all_brp_error_count,
            general_brp_error_count,
            candidate_lists,
            restoration_count: rng().random_range(0..=20),
        },
        context,
    )
    .into_response())
}

pub async fn toggle_examination_finish(
    _: CsbPoliticalGroupToggleFinishPath,
    store: CsbStore,
) -> Result<Response, AppError> {
    store.update(CsbEvent::ToggleFinish).await?;
    Ok(Redirect::to(
        &CsbPoliticalGroup::new_from_csb_store(&store)
            .after_toggle_finish_examination_path()
            .to_string(),
    )
    .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    use crate::test_utils::{response_body_string, sample_political_group};

    #[tokio::test]
    async fn political_group_renders_imported_display_name() {
        let store = CsbStore::new_for_test();
        store.data.write().imported_data.political_group = sample_political_group();
        let stream_id = store.stream_id;

        let response = overview(
            CsbPoliticalGroupPath { stream_id },
            CsbContext::new_test(),
            store,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        // The display name is used as the page title.
        let body = response_body_string(response).await;
        assert!(body.contains("Kiesraad Demo"));
    }

    #[tokio::test]
    async fn political_group_falls_back_to_placeholder_when_unnamed() {
        // A fresh store has no imported political group, so the name is unknown.
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let response = overview(
            CsbPoliticalGroupPath { stream_id },
            CsbContext::new_test(),
            store,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("?"));
    }

    #[tokio::test]
    async fn toggle_examination_finish_twice() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        // default unfinished => false
        assert!(!store.data.read().is_examination_finished);

        toggle_examination_finish(
            CsbPoliticalGroupToggleFinishPath { stream_id },
            store.clone(),
        )
        .await
        .unwrap();

        // toggle once => true
        assert!(store.data.read().is_examination_finished);

        toggle_examination_finish(
            CsbPoliticalGroupToggleFinishPath { stream_id },
            store.clone(),
        )
        .await
        .unwrap();

        // toggle twice => false
        assert!(!store.data.read().is_examination_finished);
    }

    #[tokio::test]
    async fn toggle_examination_finish_redirects() {
        let store = CsbStore::new_for_test();

        let stream_id = store.stream_id;

        let response =
            toggle_examination_finish(CsbPoliticalGroupToggleFinishPath { stream_id }, store)
                .await
                .unwrap()
                .into_response();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get("Location")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.contains(&format!("csb/examination/{stream_id}")));
    }
}
