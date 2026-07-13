use axum::{
    extract::Query,
    response::{IntoResponse, Response},
};

use crate::{
    AppError, CsbContext, CsbStore, Form, HtmlTemplate, Overlay, QueryParamState,
    csb::{
        OmissionCategory,
        examination::{
            OmissionForm,
            extractors::CsbPoliticalGroup,
            pages::{
                CsbAddOmissionPath, CsbDeleteOmissionPath, CsbOmissionOverviewPath,
                OmissionListQuery,
            },
        },
    },
    form::FormData,
};

mod urls;
mod views;

use urls::{add_url, overview_url, overview_url_for, return_path};
use views::{CsbAddOmissionTemplate, CsbOmissionOverviewTemplate, omission_views, preset_views};

/// Render the "add omission" overlay dialog.
pub async fn add_omission(
    CsbAddOmissionPath {
        stream_id,
        omission_type,
        reference,
    }: CsbAddOmissionPath,
    context: CsbContext,
    store: CsbStore,
    Query(query): Query<QueryParamState>,
    Query(OmissionListQuery { list, general }): Query<OmissionListQuery>,
) -> Result<Response, AppError> {
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    let presets = preset_views(
        omission_type,
        reference,
        list,
        general,
        &store,
        context.session.locale,
    );
    Ok(HtmlTemplate(
        CsbAddOmissionTemplate {
            form: FormData::new(),
            overlay: Overlay::new(&query),
            close_action: return_path(omission_type, reference, list, &political_group),
            presets,
            add_tab_url: add_url(stream_id, omission_type, reference, list, general),
            overview_tab_url: overview_url(stream_id, omission_type, reference, list, general),
        },
        context,
    )
    .into_response())
}

/// Render the omissions overview page for an entity: the list of omissions
/// already added, shown in the same dialog as the add-omission form but on its
/// own tab (and its own route).
pub async fn overview(
    CsbOmissionOverviewPath {
        stream_id,
        omission_type,
        reference,
    }: CsbOmissionOverviewPath,
    context: CsbContext,
    store: CsbStore,
    Query(query): Query<QueryParamState>,
    Query(OmissionListQuery { list, general }): Query<OmissionListQuery>,
) -> Result<Response, AppError> {
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    let overview_tab_url = overview_url(stream_id, omission_type, reference, list, general);

    Ok(HtmlTemplate(
        CsbOmissionOverviewTemplate {
            overlay: Overlay::new(&query),
            close_action: return_path(omission_type, reference, list, &political_group),
            omissions: omission_views(
                stream_id,
                omission_type,
                reference,
                &store,
                &overview_tab_url,
            ),
            add_tab_url: add_url(stream_id, omission_type, reference, list, general),
            overview_tab_url,
        },
        context,
    )
    .into_response())
}

/// Handle the submitted "add omission" form: validate, attach the category
/// derived from the path parameters, persist, and redirect back.
pub async fn add_omission_submit(
    CsbAddOmissionPath {
        stream_id,
        omission_type,
        reference,
    }: CsbAddOmissionPath,
    context: CsbContext,
    store: CsbStore,
    Query(query): Query<QueryParamState>,
    Query(OmissionListQuery { list, general }): Query<OmissionListQuery>,
    Form(form): Form<OmissionForm>,
) -> Result<Response, AppError> {
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    let presets = preset_views(
        omission_type,
        reference,
        list,
        general,
        &store,
        context.session.locale,
    );

    match form.validate_create() {
        Err(form_data) => Ok(HtmlTemplate(
            CsbAddOmissionTemplate {
                form: form_data,
                overlay: Overlay::new(&query),
                close_action: return_path(omission_type, reference, list, &political_group),
                presets,
                add_tab_url: add_url(stream_id, omission_type, reference, list, general),
                overview_tab_url: overview_url(stream_id, omission_type, reference, list, general),
            },
            context,
        )
        .into_response()),
        Ok(mut omission) => {
            // A general candidate omission applies to the person on every list,
            // so the list context is dropped from the persisted category even
            // though it still drives the return path below.
            let category_list = if general { None } else { list };
            omission.category =
                OmissionCategory::from_type_and_reference(omission_type, reference, category_list);
            omission.create(&store).await?;

            Ok(query.redirect_or(return_path(
                omission_type,
                reference,
                list,
                &political_group,
            )))
        }
    }
}

/// Remove a single omission and return to the overview it was removed from (the
/// `redirect_to` carried by the button, falling back to the overview derived
/// from the omission's category).
pub async fn delete_omission(
    CsbDeleteOmissionPath {
        stream_id,
        omission_id,
    }: CsbDeleteOmissionPath,
    _context: CsbContext,
    store: CsbStore,
    Query(query): Query<QueryParamState>,
) -> Result<Response, AppError> {
    let omission = store.get_omission(omission_id)?;
    let fallback = overview_url_for(&omission.category, stream_id);
    omission.delete(&store).await?;

    Ok(query.redirect_or(fallback))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    use crate::{
        candidate_lists::CandidateListId,
        csb::{Omission, OmissionType},
        persons::PersonId,
        test_utils::response_body_string,
    };

    fn sample_form() -> OmissionForm {
        OmissionForm {
            title: "Waarborgsom ontbreekt".to_string(),
            description: "De waarborgsom ontbreekt.".to_string(),
            help_text: "Please pay the deposit.".to_string(),
            recoverable: true,
        }
    }

    #[tokio::test]
    async fn add_omission_renders_csrf_and_fields() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let response = add_omission(
            CsbAddOmissionPath {
                stream_id,
                omission_type: OmissionType::PoliticalGroup,
                reference: stream_id.into(),
            },
            CsbContext::new_test(),
            store,
            Query(QueryParamState::default()),
            Query(OmissionListQuery::default()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("name=\"csrf_token\""));
        assert!(body.contains("name=\"title\""));
        assert!(body.contains("name=\"description\""));
        assert!(body.contains("name=\"help_text\""));
        // The dialog title renders with a resolved translation (the test session
        // uses the English locale).
        assert!(body.contains("Add omissions"));
        // The pill shows the short preset title, while the full description and
        // help text ride along in data attributes for the client to fill in.
        assert!(body.contains("De machtiging aanduiding ontbreekt"));
        assert!(body.contains("data-title="));
        assert!(body.contains("data-description="));
        assert!(body.contains("data-help-text="));
        assert!(body.contains("data-omission-help-text"));
        // The recoverable flag rides along on the presets and is editable through
        // a checkbox in the form.
        assert!(body.contains("data-recoverable="));
        assert!(body.contains("name=\"recoverable\""));
        // An irreparable preset ("onherstelbaar verzuim") is highlighted as an
        // error and carries `data-recoverable="false"`.
        assert!(body.contains("De aanduiding is niet geregistreerd"));
        assert!(body.contains("omission-preset-unrecoverable"));
        assert!(body.contains("data-recoverable=\"false\""));
        // No unresolved translation keys leaked through.
        assert!(!body.contains("[csb.omission"));
        // The dialog carries a two-step sidebar linking to both tabs, with the
        // add-omission form active by default and the overview on its own route.
        assert!(body.contains("steps-nav"));
        assert!(body.contains(&format!(
            "/csb/examination/{stream_id}/omission/political-group/{stream_id}/overview"
        )));
        assert!(body.contains(">Overview</a>"));
    }

    #[tokio::test]
    async fn overview_tab_lists_added_omissions_with_details() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let list = CandidateListId::new();

        // A recoverable and an irreparable omission on the same candidate list.
        Omission::new(
            OmissionCategory::CandidateList(list),
            "Waarborgsom ontbreekt".to_string(),
            "De waarborgsom ontbreekt.".to_string(),
            "Betaal de waarborgsom.".to_string(),
        )
        .create(&store)
        .await
        .unwrap();
        let mut irreparable = Omission::new(
            OmissionCategory::CandidateList(list),
            "Aanduiding niet geregistreerd".to_string(),
            "De aanduiding is niet geregistreerd.".to_string(),
            String::new(),
        );
        irreparable.recoverable = false;
        irreparable.create(&store).await.unwrap();

        let response = overview(
            CsbOmissionOverviewPath {
                stream_id,
                omission_type: OmissionType::CandidateList,
                reference: list.into(),
            },
            CsbContext::new_test(),
            store,
            Query(QueryParamState::default()),
            Query(OmissionListQuery::default()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        // The overview shows each omission's title, description and help text.
        assert!(body.contains("Waarborgsom ontbreekt"));
        assert!(body.contains("De waarborgsom ontbreekt."));
        assert!(body.contains("Betaal de waarborgsom."));
        // The recoverable flag is surfaced per omission.
        assert!(body.contains(">Recoverable</span>"));
        assert!(body.contains(">Not recoverable</span>"));
        assert!(body.contains("omission-item-unrecoverable"));
        // The overview drops the add-omission form (no description field, no
        // submit/save button).
        assert!(!body.contains("data-omission-description"));
        assert!(!body.contains("value=\"save\""));
        // The sidebar still links back to the add-omission form.
        assert!(body.contains("steps-nav"));
        assert!(body.contains(&format!(
            "/csb/examination/{stream_id}/omission/candidate-list/{list}\""
        )));
        // Each omission carries a remove button targeting its delete action.
        assert!(body.contains(&format!("/csb/examination/{stream_id}/delete-omission/")));
        assert!(body.contains(">Remove</button>"));
    }

    #[tokio::test]
    async fn delete_omission_removes_it_and_redirects_to_the_overview() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let list = CandidateListId::new();

        let omission = Omission::new(
            OmissionCategory::CandidateList(list),
            "Waarborgsom ontbreekt".to_string(),
            "De waarborgsom ontbreekt.".to_string(),
            String::new(),
        );
        omission.create(&store).await.unwrap();
        let omission_id = omission.id;
        assert_eq!(store.get_candidate_list_omissions(list).len(), 1);

        let response = delete_omission(
            CsbDeleteOmissionPath {
                stream_id,
                omission_id,
            },
            CsbContext::new_test(),
            store.clone(),
            Query(QueryParamState::default()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        // The omission is gone...
        assert!(store.get_candidate_list_omissions(list).is_empty());
        // ...and without an explicit redirect we fall back to the candidate list
        // overview it belonged to.
        let location = response
            .headers()
            .get("Location")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.contains(&format!(
            "/csb/examination/{stream_id}/omission/candidate-list/{list}/overview"
        )));
    }

    #[tokio::test]
    async fn delete_omission_honours_the_redirect_to() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let omission = Omission::new(
            OmissionCategory::General,
            "Deposit missing".to_string(),
            "The deposit is missing.".to_string(),
            String::new(),
        );
        omission.create(&store).await.unwrap();
        let omission_id = omission.id;

        let response = delete_omission(
            CsbDeleteOmissionPath {
                stream_id,
                omission_id,
            },
            CsbContext::new_test(),
            store.clone(),
            Query(QueryParamState::redirect_to("/back/here".to_string())),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert!(store.get_general_omissions().is_empty());
        let location = response
            .headers()
            .get("Location")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.starts_with("/back/here"));
    }

    #[tokio::test]
    async fn overview_shows_empty_state_without_omissions() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let response = overview(
            CsbOmissionOverviewPath {
                stream_id,
                omission_type: OmissionType::PoliticalGroup,
                reference: stream_id.into(),
            },
            CsbContext::new_test(),
            store,
            Query(QueryParamState::default()),
            Query(OmissionListQuery::default()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("No omissions have been added yet."));
    }

    #[tokio::test]
    async fn add_candidate_list_omission_persists_the_right_category() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let list = CandidateListId::new();
        let context = CsbContext::new_test();
        let form = sample_form();

        let response = add_omission_submit(
            CsbAddOmissionPath {
                stream_id,
                omission_type: OmissionType::CandidateList,
                reference: list.into(),
            },
            context,
            store.clone(),
            Query(QueryParamState::default()),
            Query(OmissionListQuery::default()),
            Form(form),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        // The dialog redirects back to the candidate list it was opened from.
        let location = response
            .headers()
            .get("Location")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.contains(&format!("/list/{list}")));

        let omission = store.get_omission_for_test();
        assert_eq!(omission.title, "Waarborgsom ontbreekt");
        assert_eq!(omission.description, "De waarborgsom ontbreekt.");
        assert!(matches!(
            omission.category,
            OmissionCategory::CandidateList(id) if id == list
        ));
    }

    #[tokio::test]
    async fn add_general_omission_persists_general_category() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let context = CsbContext::new_test();
        let form = sample_form();

        add_omission_submit(
            CsbAddOmissionPath {
                stream_id,
                omission_type: OmissionType::PoliticalGroup,
                reference: stream_id.into(),
            },
            context,
            store.clone(),
            Query(QueryParamState::default()),
            Query(OmissionListQuery::default()),
            Form(form),
        )
        .await
        .unwrap();

        assert_eq!(store.get_general_omissions().len(), 1);
    }

    #[tokio::test]
    async fn add_omission_persists_the_recoverable_flag() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let context = CsbContext::new_test();
        // An unchecked "recoverable" checkbox submits nothing, marking the
        // omission irreparable.
        let mut form = sample_form();
        form.recoverable = false;

        add_omission_submit(
            CsbAddOmissionPath {
                stream_id,
                omission_type: OmissionType::PoliticalGroup,
                reference: stream_id.into(),
            },
            context,
            store.clone(),
            Query(QueryParamState::default()),
            Query(OmissionListQuery::default()),
            Form(form),
        )
        .await
        .unwrap();

        let omission = store.get_omission_for_test();
        assert!(!omission.recoverable);
    }

    #[tokio::test]
    async fn add_omission_invalid_form_rerenders() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let context = CsbContext::new_test();
        let mut form = sample_form();
        // An empty description is invalid.
        form.description = String::new();

        let response = add_omission_submit(
            CsbAddOmissionPath {
                stream_id,
                omission_type: OmissionType::PoliticalGroup,
                reference: stream_id.into(),
            },
            context,
            store.clone(),
            Query(QueryParamState::default()),
            Query(OmissionListQuery::default()),
            Form(form),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(store.get_general_omissions().is_empty());
    }

    #[tokio::test]
    async fn candidate_dialog_interpolates_candidate_placeholders() {
        use crate::test_utils::{sample_candidate_list, sample_person};

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        // Seed a candidate at position 1 of a list.
        let person = sample_person(PersonId::new());
        let person_id = person.id;
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.candidates = vec![person_id];
        store.set_person(person);
        store.set_candidate_list(list);

        let response = add_omission(
            CsbAddOmissionPath {
                stream_id,
                omission_type: OmissionType::Candidate,
                reference: person_id.into(),
            },
            CsbContext::new_test(),
            store,
            Query(QueryParamState::default()),
            Query(OmissionListQuery {
                list: Some(list_id),
                general: false,
            }),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        // The candidate's name and position are interpolated into the preset.
        assert!(body.contains("Kandidaat nr. 1, Jansen, H.A.H.A. (Henk)"));
        // The unresolved token is left for the committee to fill in manually.
        assert!(body.contains("{designation}"));
        assert!(!body.contains("{candidate_name}"));
    }

    #[tokio::test]
    async fn person_dialog_offers_the_person_presets() {
        use crate::test_utils::{sample_candidate_list, sample_person};

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let person = sample_person(PersonId::new());
        let person_id = person.id;
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.candidates = vec![person_id];
        store.set_person(person);
        store.set_candidate_list(list);

        // `general` opens the person dialog, which draws from the "person" preset
        // set rather than the candidate-on-this-list set.
        let response = add_omission(
            CsbAddOmissionPath {
                stream_id,
                omission_type: OmissionType::Candidate,
                reference: person_id.into(),
            },
            CsbContext::new_test(),
            store,
            Query(QueryParamState::default()),
            Query(OmissionListQuery {
                list: Some(list_id),
                general: true,
            }),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        // A person preset shows up...
        assert!(body.contains("Kopie ID ontbreekt"));
        // ...while a preset scoped to the listing on the list does not.
        assert!(!body.contains("onjuiste nadere aanduidingen"));
        // The list context still resolves the candidate placeholders.
        assert!(body.contains("Jansen"));
        assert!(!body.contains("{candidate_name}"));
    }

    #[tokio::test]
    async fn candidate_position_is_scoped_to_the_referenced_list() {
        use crate::test_utils::{sample_candidate_list, sample_person};

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        // The same candidate sits at different positions on two lists.
        let person = sample_person(PersonId::new());
        let person_id = person.id;
        store.set_person(person);

        let first_list_id = CandidateListId::new();
        let mut first_list = sample_candidate_list(first_list_id);
        first_list.candidates = vec![person_id];
        store.set_candidate_list(first_list);

        let second_list_id = CandidateListId::new();
        let mut second_list = sample_candidate_list(second_list_id);
        second_list.candidates = vec![PersonId::new(), person_id];
        store.set_candidate_list(second_list);

        // Opening the dialog for the second list resolves position 2, not 1.
        let response = add_omission(
            CsbAddOmissionPath {
                stream_id,
                omission_type: OmissionType::Candidate,
                reference: person_id.into(),
            },
            CsbContext::new_test(),
            store,
            Query(QueryParamState::default()),
            Query(OmissionListQuery {
                list: Some(second_list_id),
                general: false,
            }),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Kandidaat nr. 2, Jansen, H.A.H.A. (Henk)"));
        assert!(!body.contains("Kandidaat nr. 1, Jansen"));
    }

    #[tokio::test]
    async fn add_candidate_omission_persists_the_list() {
        use crate::test_utils::{sample_candidate_list, sample_person};

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let context = CsbContext::new_test();
        let form = sample_form();

        let person = sample_person(PersonId::new());
        let person_id = person.id;
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.candidates = vec![person_id];
        store.set_person(person);
        store.set_candidate_list(list);

        let response = add_omission_submit(
            CsbAddOmissionPath {
                stream_id,
                omission_type: OmissionType::Candidate,
                reference: person_id.into(),
            },
            context,
            store.clone(),
            Query(QueryParamState::default()),
            Query(OmissionListQuery {
                list: Some(list_id),
                general: false,
            }),
            Form(form),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        // The dialog redirects back to the candidate detail page it was opened
        // from, scoped to the list the candidate was examined on.
        let location = response
            .headers()
            .get("Location")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.contains(&format!("/list/{list_id}/candidate/{person_id}")));

        let omission = store.get_omission_for_test();
        assert!(matches!(
            omission.category,
            OmissionCategory::Candidate { person, list }
                if person == person_id && list == Some(list_id)
        ));
    }

    #[tokio::test]
    async fn add_person_omission_persists_a_general_candidate_category() {
        use crate::test_utils::{sample_candidate_list, sample_person};

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let context = CsbContext::new_test();
        let form = sample_form();

        let person = sample_person(PersonId::new());
        let person_id = person.id;
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.candidates = vec![person_id];
        store.set_person(person);
        store.set_candidate_list(list);

        let response = add_omission_submit(
            CsbAddOmissionPath {
                stream_id,
                omission_type: OmissionType::Candidate,
                reference: person_id.into(),
            },
            context,
            store.clone(),
            Query(QueryParamState::default()),
            // `general` marks the omission as applying to the person on every
            // list; `list` is still carried so we return to the page we're on.
            Query(OmissionListQuery {
                list: Some(list_id),
                general: true,
            }),
            Form(form),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        // Even a general omission returns to the candidate detail page it was
        // opened from.
        let location = response
            .headers()
            .get("Location")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.contains(&format!("/list/{list_id}/candidate/{person_id}")));

        // The persisted category applies to the whole person, not this list.
        let omission = store.get_omission_for_test();
        assert!(matches!(
            omission.category,
            OmissionCategory::Candidate { person, list }
                if person == person_id && list.is_none()
        ));
    }
}
