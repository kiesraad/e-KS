use askama::Template;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::{TypedHeader, headers};

use crate::{
    AppError, Context, CsbContext,
    CsbEvent::{self},
    CsbStore, HtmlTemplate,
    csb::examination::{
        extractors::CsbPoliticalGroup,
        pages::{CsbPoliticalGroupPath, CsbPoliticalGroupToggleFinishPath},
        structs::CsbCandidateList,
    },
    filters,
};

#[derive(Template)]
#[template(path = "csb/examination/pages/political_group.html")]
struct CsbPoliticalGroupTemplate {
    political_group: CsbPoliticalGroup,
    all_brp_error_count: usize,
    candidate_lists: Vec<CsbCandidateList>,
    political_group_omission_count: usize,
    restoration_count: usize,
}

/// Render the placeholder political group overview page.
pub async fn overview(
    _: CsbPoliticalGroupPath,
    context: CsbContext,
    store: CsbStore,
) -> Result<Response, AppError> {
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    let candidate_lists = store
        .get_candidate_lists()
        .into_iter()
        .map(CsbCandidateList::placeholder)
        .collect::<Vec<_>>();
    let all_brp_error_count = candidate_lists
        .iter()
        .map(|cl| cl.brp_error_count)
        .sum::<usize>();
    let political_group_omission_count = store.get_political_group_omissions().len();

    Ok(HtmlTemplate(
        CsbPoliticalGroupTemplate {
            political_group,
            all_brp_error_count,
            candidate_lists,
            political_group_omission_count,
            restoration_count: store.get_omission_count(),
        },
        context,
    )
    .into_response())
}

pub async fn toggle_examination_finish(
    _: CsbPoliticalGroupToggleFinishPath,
    TypedHeader(referer): TypedHeader<headers::Referer>,
    store: CsbStore,
) -> Result<Response, AppError> {
    let finished = store.is_examination_finished();
    store.update(CsbEvent::SetFinished(!finished)).await?;
    Ok(Redirect::to(&referer.to_string()).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_extra::headers::Referer;

    use crate::test_utils::{response_body_string, sample_political_group};

    #[tokio::test]
    async fn political_group_renders_imported_display_name() {
        let store = CsbStore::new_for_test();
        store.set_political_group(sample_political_group());
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
        assert!(body.contains("Blanco"));
    }

    #[tokio::test]
    async fn renders_political_group_omission_count_badge() {
        use crate::csb::{Omission, OmissionCategory};

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        Omission::new(
            OmissionCategory::PoliticalGroup,
            "Deposit missing".to_string(),
            "The deposit has not been paid.".to_string(),
            String::new(),
        )
        .create(&store)
        .await
        .unwrap();

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
        assert!(body.contains("omission-badge"));
        assert!(body.contains("1 omission"));
    }

    #[tokio::test]
    async fn toggle_examination_finish_twice() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        // default unfinished => false
        assert!(!store.is_examination_finished());

        toggle_examination_finish(
            CsbPoliticalGroupToggleFinishPath { stream_id },
            TypedHeader(Referer::from_static("test_referer")),
            store.clone(),
        )
        .await
        .unwrap();

        // toggle once => true
        assert!(store.is_examination_finished());

        toggle_examination_finish(
            CsbPoliticalGroupToggleFinishPath { stream_id },
            TypedHeader(Referer::from_static("test_referer")),
            store.clone(),
        )
        .await
        .unwrap();

        // toggle twice => false
        assert!(!store.is_examination_finished());
    }

    #[tokio::test]
    async fn toggle_examination_finish_redirects_to_referer_header() {
        let store = CsbStore::new_for_test();
        let example_url = "http://example.com";
        let stream_id = store.stream_id;

        let response = toggle_examination_finish(
            CsbPoliticalGroupToggleFinishPath { stream_id },
            TypedHeader(Referer::from_static(example_url)),
            store,
        )
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
        assert_eq!(location, "http://example.com");
    }
}
