use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{
    AppError, Context, CsbContext, CsbStore, ElectoralDistrict, HtmlTemplate,
    candidate_lists::CandidateListId,
    csb::{
        Omission,
        examination::{extractors::CsbPoliticalGroup, pages::CsbCandidatePath},
    },
    filters,
    persons::Person,
};

#[derive(Template)]
#[template(path = "csb/examination/pages/candidate.html")]
struct CsbCandidateTemplate {
    political_group: CsbPoliticalGroup,
    list_id: CandidateListId,
    electoral_districts: Vec<ElectoralDistrict>,
    candidate: Person,
    position: Option<usize>,
    candidate_omissions: Vec<Omission>,
}

pub async fn overview(
    CsbCandidatePath {
        list_id, person_id, ..
    }: CsbCandidatePath,
    context: CsbContext,
    store: CsbStore,
) -> Result<Response, AppError> {
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    let candidate = store
        .get_person(person_id)
        .ok_or(AppError::GenericNotFound)?;
    let position = store.candidate_position(list_id, person_id);
    let electoral_districts = store
        .get_candidate_list(list_id)
        .map(|list| list.electoral_districts)
        .ok_or(AppError::GenericNotFound)?;
    let candidate_omissions = store.get_candidate_omissions(person_id);

    Ok(HtmlTemplate(
        CsbCandidateTemplate {
            political_group,
            list_id,
            electoral_districts,
            candidate,
            position,
            candidate_omissions,
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
    async fn renders_candidate_details_and_add_omission_buttons() {
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
            CsbCandidatePath {
                stream_id,
                list_id,
                person_id,
            },
            CsbContext::new_test(),
            store,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        // The candidate's imported details render.
        assert!(body.contains("Jansen"));
        assert!(body.contains("Juinen"));
        // Both add-omission buttons target the candidate omission dialog,
        // carrying the list so the candidate's position can be resolved.
        assert!(body.contains(&format!(
            "/csb/examination/{stream_id}/omission/candidate/{person_id}"
        )));
        assert!(body.contains(&format!("list={list_id}")));
        // One button adds an omission for the person on every list (general),
        // the other for the candidate on this specific list. Only the general
        // one carries the `general` flag.
        assert!(body.contains("general=true"));
        // The header shows the electoral districts of the candidate's list
        // (the sample list covers Utrecht).
        assert!(body.contains("Electoral districts"));
        assert!(body.contains("Utrecht"));
    }

    #[tokio::test]
    async fn renders_added_candidate_omissions_as_badges() {
        use crate::csb::OmissionCategory;

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let person = sample_person(PersonId::new());
        let person_id = person.id;
        let list_id = CandidateListId::new();
        store.set_person(person);
        store.set_candidate_list(sample_candidate_list(list_id));

        Omission::new(
            OmissionCategory::Candidate {
                person: person_id,
                list: Some(list_id),
            },
            "Missing consent".to_string(),
            "The declaration of consent is missing.".to_string(),
            String::new(),
        )
        .create(&store)
        .await
        .unwrap();

        let response = overview(
            CsbCandidatePath {
                stream_id,
                list_id,
                person_id,
            },
            CsbContext::new_test(),
            store,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("omission-badge"));
        assert!(body.contains("Missing consent"));
    }

    #[tokio::test]
    async fn returns_not_found_for_unknown_candidate() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let result = overview(
            CsbCandidatePath {
                stream_id,
                list_id: CandidateListId::new(),
                person_id: PersonId::new(),
            },
            CsbContext::new_test(),
            store,
        )
        .await;

        assert!(matches!(result, Err(AppError::GenericNotFound)));
    }

    #[tokio::test]
    async fn returns_not_found_for_unknown_list() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        // A known candidate but an unknown list: the person lookup succeeds,
        // so the handler fails when resolving the list's electoral districts.
        let person = sample_person(PersonId::new());
        let person_id = person.id;
        store.set_person(person);

        let result = overview(
            CsbCandidatePath {
                stream_id,
                list_id: CandidateListId::new(),
                person_id,
            },
            CsbContext::new_test(),
            store,
        )
        .await;

        assert!(matches!(result, Err(AppError::GenericNotFound)));
    }
}
