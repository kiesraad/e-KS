use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{
    AppError, Context, CsbContext, CsbStore, ElectoralDistrict, HtmlTemplate,
    csb::examination::{
        extractors::CsbPoliticalGroup,
        pages::CsbCandidatePath,
        structs::{PaperCorrected, PaperCorrectedPersonDetails},
    },
    filters,
    projection::WithCorrections,
    structs::{candidate_lists::CandidateListId, csb::Omission, persons::Person},
};

#[derive(Template)]
#[template(path = "csb/examination/pages/candidate.html")]
struct CsbCandidateTemplate {
    political_group: CsbPoliticalGroup,
    list_id: CandidateListId,
    electoral_districts: Vec<ElectoralDistrict>,
    candidate: Person,
    details: PaperCorrectedPersonDetails,
    position: PaperCorrected,
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

    let imported = store.get_person(person_id, WithCorrections::None);
    let corrected = store.get_person(person_id, WithCorrections::Paper);
    let csb_corrected = store.get_person(person_id, WithCorrections::All);
    let candidate = imported
        .clone()
        .or_else(|| corrected.clone())
        .ok_or(AppError::GenericNotFound)?;
    let details = PaperCorrectedPersonDetails::new(
        imported.as_ref(),
        corrected.as_ref(),
        csb_corrected.as_ref(),
        context.session.locale,
    );
    let position = PaperCorrected::new(
        store
            .get_candidate_position(list_id, person_id, WithCorrections::None)
            .map(|p| p.to_string())
            .unwrap_or_default(),
        store
            .get_candidate_position(list_id, person_id, WithCorrections::Paper)
            .map(|p| p.to_string())
            .unwrap_or_default(),
    );
    // The corrected electoral districts take precedence over the imported ones.
    let electoral_districts = store
        .get_candidate_list(list_id, WithCorrections::All)
        .map(|list| list.electoral_districts)
        .ok_or(AppError::GenericNotFound)?;
    let candidate_omissions = store.get_candidate_omissions(person_id);

    Ok(HtmlTemplate(
        CsbCandidateTemplate {
            political_group,
            list_id,
            electoral_districts,
            candidate,
            details,
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
        structs::{csb::OmissionCategory, persons::PersonId},
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
        store.add_person(person);
        store.add_candidate_list(list);

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
        // The add-omission button targets the candidate omission dialog,
        // carrying the list so the candidate's position can be resolved.
        assert!(body.contains(&format!(
            "/csb/examination/{stream_id}/omission/candidate/{person_id}"
        )));
        assert!(body.contains(&format!("list={list_id}")));
        // The header shows the electoral districts of the candidate's list
        // (the sample list covers Utrecht).
        assert!(body.contains("Electoral districts"));
        assert!(body.contains("Utrecht"));
        // date of birth formatted correctly
        assert!(body.contains("01-02-1990"))
    }

    #[tokio::test]
    async fn shows_bsn_house_number_addition_and_representative_corrections() {
        use crate::{
            structs::{common::BsnOrNoneConfirmed, persons::Representative},
            test_utils::{sample_dutch_address, sample_full_name},
        };

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let mut person = sample_person(PersonId::new());
        person.personal_data.bsn = Some(BsnOrNoneConfirmed::Bsn("999995972".parse().unwrap()));
        person.representative = Some(Representative {
            name: sample_full_name(None, "Gemachtigde", None, "G.G."),
            address: sample_dutch_address("Den Haag", "2513 AA", "1", "B", "Plein"),
        });
        let person_id = person.id;
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.candidates = vec![person_id];
        store.add_person(person.clone());
        store.add_candidate_list(list);

        // The corrections change the BSN, the candidate's house number
        // addition and the representative's last name.
        let mut corrected = person;
        corrected.personal_data.bsn = Some(BsnOrNoneConfirmed::NoneConfirmed);
        corrected.address.house_number_addition = Some("C".parse().unwrap());
        corrected.representative.as_mut().unwrap().name =
            sample_full_name(None, "Opvolger", None, "G.G.");
        store
            .data
            .write()
            .paper_corrected_data
            .persons
            .insert(person_id, corrected);

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
        // The corrected BSN renders next to the struck-through imported one.
        assert!(body.contains(r#"<s class="imported-value">999995972</s>"#));
        // The corrected house number addition is highlighted.
        assert!(body.contains(r#"<strong class="paper-corrected-value">C</strong>"#));
        // The representative table renders with the corrected name.
        assert!(body.contains("Authorised person"));
        assert!(body.contains("Gemachtigde"));
        assert!(body.contains("Opvolger"));
    }

    #[tokio::test]
    async fn shows_corrected_electoral_districts() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let person = sample_person(PersonId::new());
        let person_id = person.id;
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.candidates = vec![person_id];
        store.add_person(person);
        store.add_candidate_list(list.clone());
        let mut corrected = list;
        corrected.electoral_districts = vec![ElectoralDistrict::Groningen];
        store.set_paper_corrected_candidate_list(corrected);

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
        // The corrected districts replace the imported ones.
        assert!(body.contains("Groningen"));
        assert!(!body.contains("Utrecht"));
    }

    #[tokio::test]
    async fn renders_the_corrected_position_when_it_differs() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let person = sample_person(PersonId::new());
        let person_id = person.id;
        let other = sample_person(PersonId::new());
        let other_id = other.id;
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.candidates = vec![person_id, other_id];
        store.add_person(person);
        store.add_person(other);
        store.add_candidate_list(list.clone());

        // The corrections move the candidate from position 1 to 2.
        list.candidates = vec![other_id, person_id];
        store.set_paper_corrected_candidate_list(list);

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
        // The imported position renders struck through, followed by the
        // corrected position badge.
        assert!(
            body.contains(
                r#"<s class="badge position-badge imported-value candidate-number">1</s>"#
            )
        );
        assert!(body.contains("paper-corrected-value"));
    }

    #[tokio::test]
    async fn renders_added_candidate_omissions_as_badges() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let person = sample_person(PersonId::new());
        let person_id = person.id;
        let list_id = CandidateListId::new();
        store.add_person(person);
        store.add_candidate_list(sample_candidate_list(list_id));

        Omission::new(
            OmissionCategory::Candidate {
                person: person_id,
                lists: vec![list_id],
            },
            "Missing consent".parse().unwrap(),
            "The declaration of consent is missing.".parse().unwrap(),
            None,
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
        assert!(body.contains("restoration-tag"));
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
        store.add_person(person);

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
