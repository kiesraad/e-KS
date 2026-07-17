use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{
    AppError, Context, CsbContext, CsbStore, ElectoralDistrict, HtmlTemplate,
    candidate_lists::CandidateListId,
    csb::{
        Omission,
        examination::{
            extractors::CsbPoliticalGroup, pages::CsbCandidateListPath, structs::CsbCandidate,
        },
    },
    filters,
};

#[derive(Template)]
#[template(path = "csb/examination/pages/candidate_list.html")]
struct CsbCandidateListTemplate {
    political_group: CsbPoliticalGroup,
    list_id: CandidateListId,
    electoral_districts: Vec<ElectoralDistrict>,
    candidates: Vec<CsbCandidate>,
    omissions: Vec<Omission>,
    restoration_count: usize
}

pub async fn overview(
    CsbCandidateListPath { list_id, .. }: CsbCandidateListPath,
    context: CsbContext,
    store: CsbStore,
) -> Result<Response, AppError> {
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    let list = store
        .get_candidate_list(list_id)
        .ok_or(AppError::GenericNotFound)?;

    let candidates = list
        .candidates
        .iter()
        .enumerate()
        .filter_map(|(index, person_id)| {
            store
                .get_person(*person_id)
                .map(|person| CsbCandidate::placeholder(person, index + 1))
        })
        .collect::<Vec<_>>();

    let omissions = store.get_candidate_list_omissions(list_id);

    Ok(HtmlTemplate(
        CsbCandidateListTemplate {
            political_group,
            list_id,
            electoral_districts: list.electoral_districts,
            candidates,
            omissions,
            restoration_count: store.get_omission_count()
        },
        context,
    )
    .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    use crate::{
        persons::PersonId,
        test_utils::{response_body_string, sample_candidate_list, sample_person},
    };

    #[tokio::test]
    async fn renders_candidates_with_brp_column_and_add_omission_button() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let person = sample_person(PersonId::new());
        let person_id = person.id;
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.candidates = vec![person_id];
        store.set_person(person);
        store.set_candidate_list(list);

        let response = overview(
            CsbCandidateListPath { stream_id, list_id },
            CsbContext::new_test(),
            store,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        // The candidate is listed and the BRP errors column header renders.
        assert!(body.contains("Jansen"));
        assert!(body.contains("BRP errors"));
        // The add omission button links to the candidate list omission dialog.
        assert!(body.contains(&format!(
            "/csb/examination/{stream_id}/omission/candidate-list/"
        )));
        // Rows link to the candidate detail page.
        assert!(body.contains(&format!(
            "/csb/examination/{stream_id}/list/{list_id}/candidate/{person_id}"
        )));
        // The list's electoral districts are shown (the sample list covers Utrecht).
        assert!(body.contains("Electoral districts"));
        assert!(body.contains("Utrecht"));
    }

    #[tokio::test]
    async fn renders_added_candidate_list_omissions_as_badges() {
        use crate::csb::{Omission, OmissionCategory};

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let list_id = CandidateListId::new();
        store.set_candidate_list(sample_candidate_list(list_id));
        Omission::new(
            OmissionCategory::CandidateList(list_id),
            "Too many candidates".to_string(),
            "The list holds more candidates than allowed.".to_string(),
            String::new(),
        )
        .create(&store)
        .await
        .unwrap();

        let response = overview(
            CsbCandidateListPath { stream_id, list_id },
            CsbContext::new_test(),
            store,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("omission-badge"));
        // The badge shows the short title, not the long description.
        assert!(body.contains("Too many candidates"));
        assert!(!body.contains("The list holds more candidates than allowed."));
    }

    #[tokio::test]
    async fn returns_not_found_for_unknown_list() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let result = overview(
            CsbCandidateListPath {
                stream_id,
                list_id: CandidateListId::new(),
            },
            CsbContext::new_test(),
            store,
        )
        .await;

        assert!(matches!(result, Err(AppError::GenericNotFound)));
    }
}
