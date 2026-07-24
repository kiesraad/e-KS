use askama::Template;
use axum::{
    extract::Query,
    response::{IntoResponse, Response},
};

use crate::{
    AppError, Context, CsbContext,
    CsbEvent::{self},
    CsbStore, HtmlTemplate, QueryParamState,
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
    declarations_of_support_omission_count: usize,
    declarations_of_support_card_path: String,
    restoration_count: usize,
}

/// Render the placeholder political group overview page.
pub async fn overview(
    _: CsbPoliticalGroupPath,
    context: CsbContext,
    store: CsbStore,
) -> Result<Response, AppError> {
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    let corrected_store = store.paper_corrected();
    // The cards show the paper-corrected lists: lists deleted by the
    // corrections are hidden, and the corrected candidates and electoral
    // districts take precedence over the imported ones.
    let mut candidate_lists = store
        .get_imported_candidate_lists()
        .into_iter()
        .filter_map(|list| {
            let corrected = corrected_store.get_candidate_list(list.id).ok()?;
            Some(CsbCandidateList::placeholder(corrected))
        })
        .collect::<Vec<_>>();
    // Lists only present in the paper-corrected projection were added during
    // paper corrections; they get a card too.
    candidate_lists.extend(
        corrected_store
            .get_candidate_lists()
            .into_iter()
            .filter(|list| store.get_imported_candidate_list(list.id).is_none())
            .map(CsbCandidateList::paper_added),
    );
    let all_brp_error_count = candidate_lists
        .iter()
        .map(|cl| cl.brp_error_count)
        .sum::<usize>();
    let political_group_omission_count = store.get_political_group_omissions().len();
    let declarations_of_support_omission_count =
        store.get_all_declarations_of_support_omissions().len();
    let declarations_of_support_card_path = if declarations_of_support_omission_count == 0 {
        political_group
            .add_declarations_of_support_omission_path()
            .to_string()
    } else {
        political_group
            .manage_declarations_of_support_omissions_path()
            .to_string()
    };

    Ok(HtmlTemplate(
        CsbPoliticalGroupTemplate {
            political_group,
            all_brp_error_count,
            candidate_lists,
            political_group_omission_count,
            declarations_of_support_omission_count,
            declarations_of_support_card_path,
            restoration_count: store.get_omission_count(),
        },
        context,
    )
    .into_response())
}

pub async fn toggle_examination_finish(
    _: CsbPoliticalGroupToggleFinishPath,
    Query(query): Query<QueryParamState>,
    store: CsbStore,
) -> Result<Response, AppError> {
    let finished = store.is_examination_finished();
    store.update(CsbEvent::SetFinished(!finished)).await?;
    Ok(query.redirect_or(CsbPoliticalGroup::new_from_csb_store(&store).examination_path()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    use crate::{
        structs::csb::{Omission, OmissionCategory},
        test_utils::{response_body_string, sample_political_group},
    };

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
        // The paper corrections card posts to the start route.
        assert!(body.contains(&format!("/csb/examination/{stream_id}/paper-corrections")));
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
    async fn renders_card_for_list_added_in_paper_corrections() {
        use crate::{structs::candidate_lists::CandidateListId, test_utils::sample_candidate_list};

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let list_id = CandidateListId::new();
        store.set_paper_corrected_candidate_list(sample_candidate_list(list_id));

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
        // The paper-added list gets a card, marked as added on paper, linking
        // to its examination page.
        assert!(body.contains("Added during paper corrections"));
        assert!(body.contains(&format!("/csb/examination/{stream_id}/list/{list_id}")));
    }

    #[tokio::test]
    async fn hides_card_for_list_deleted_in_paper_corrections() {
        use crate::{structs::candidate_lists::CandidateListId, test_utils::sample_candidate_list};

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let list_id = CandidateListId::new();
        // An imported list without a corrected counterpart was deleted on paper.
        store
            .data
            .write()
            .imported_data
            .candidate_lists
            .insert(list_id, sample_candidate_list(list_id));

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
        assert!(!body.contains(&format!("/csb/examination/{stream_id}/list/{list_id}")));
    }

    #[tokio::test]
    async fn card_shows_corrected_electoral_districts() {
        use crate::{
            ElectoralDistrict, structs::candidate_lists::CandidateListId,
            test_utils::sample_candidate_list,
        };

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let list_id = CandidateListId::new();
        store.add_candidate_list(sample_candidate_list(list_id));
        let mut corrected = sample_candidate_list(list_id);
        corrected.electoral_districts = vec![ElectoralDistrict::GR];
        store.set_paper_corrected_candidate_list(corrected);

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
        // The card shows the corrected districts, not the imported ones.
        assert!(body.contains("Groningen"));
        assert!(!body.contains("Utrecht"));
    }

    #[tokio::test]
    async fn card_shows_corrected_candidate_count() {
        use crate::{
            structs::{candidate_lists::CandidateListId, persons::PersonId},
            test_utils::sample_candidate_list,
        };

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.candidates = vec![PersonId::new()];
        store.add_candidate_list(list.clone());
        let mut corrected = list;
        corrected.candidates.push(PersonId::new());
        store.set_paper_corrected_candidate_list(corrected);

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
        // The card counts the corrected candidates, not the imported ones.
        assert!(body.contains("<strong>2</strong> candidates"));
    }

    #[tokio::test]
    async fn renders_political_group_omission_count_badge() {
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
            Query(QueryParamState::default()),
            store.clone(),
        )
        .await
        .unwrap();

        // toggle once => true
        assert!(store.is_examination_finished());

        toggle_examination_finish(
            CsbPoliticalGroupToggleFinishPath { stream_id },
            Query(QueryParamState::default()),
            store.clone(),
        )
        .await
        .unwrap();

        // toggle twice => false
        assert!(!store.is_examination_finished());
    }

    #[tokio::test]
    async fn toggle_examination_finish_honours_the_redirect_to() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let response = toggle_examination_finish(
            CsbPoliticalGroupToggleFinishPath { stream_id },
            Query(QueryParamState::redirect_to("/back/here".to_string())),
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
        assert!(location.starts_with("/back/here"));
    }

    #[tokio::test]
    async fn toggle_examination_finish_redirects_to_examination_by_default() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let response = toggle_examination_finish(
            CsbPoliticalGroupToggleFinishPath { stream_id },
            Query(QueryParamState::default()),
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
        assert!(location.contains(&format!("csb/examination/{stream_id}")));
    }
}
