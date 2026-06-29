use askama::Template;
use axum::response::{IntoResponse, Response};
use rand::{RngExt, rng};

use crate::{
    AppError, Context, CsbContext, CsbStore, ElectionConfig, HtmlTemplate,
    csb::examination::{pages::CsbPoliticalGroupPath, structs::CsbCandidateList},
    filters,
};

#[derive(Template)]
#[template(path = "examination/pages/political_group.html")]
struct CsbPoliticalGroupTemplate {
    // TODO make election part of CsbContext?
    election: ElectionConfig,
    political_group_name: String,
    all_brp_error_count: usize,
    candidate_lists: Vec<CsbCandidateList>,
    general_brp_error_count: usize,
}

/// Render the placeholder political group overview page.
pub async fn overview(
    _: CsbPoliticalGroupPath,
    context: CsbContext,
    store: CsbStore,
) -> Result<Response, AppError> {
    let data = &store.data.read().imported_data;
    let election = store.election;
    let political_group_name = data
        .political_group
        .display_name
        .as_ref()
        // TODO figure out what to do with blanco lijsten, see #870
        .map_or("?".to_string(), |dn| dn.to_string());
    let general_brp_error_count = rng().random_range(0..=2);
    let candidate_lists = data
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
            election,
            political_group_name,
            all_brp_error_count,
            general_brp_error_count,
            candidate_lists,
        },
        context,
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
}
