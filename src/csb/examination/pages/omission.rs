use askama::Template;
use axum::{
    extract::Query,
    response::{IntoResponse, Response},
};

use uuid::Uuid;

use crate::{
    AppError, Context, CsbContext, CsbStore, Form, HtmlTemplate, Locale, Overlay, QueryParamState,
    candidate_lists::CandidateListId,
    csb::{
        OmissionCategory, OmissionPlaceholders, OmissionType,
        examination::{
            OmissionForm,
            extractors::CsbPoliticalGroup,
            pages::{CsbAddOmissionPath, OmissionListQuery},
        },
    },
    filters,
    form::FormData,
    persons::PersonId,
};

#[derive(Template)]
#[template(path = "csb/examination/pages/omission.html")]
struct CsbAddOmissionTemplate {
    form: FormData<OmissionForm>,
    overlay: Overlay,
    /// Where the close button and the post-save redirect return to.
    close_action: String,
    /// Quick-fill suggestions for this type, with placeholders interpolated.
    presets: Vec<PresetView>,
}

/// A preset shown in the dialog, with `{token}` placeholders in its description
/// already filled from the referenced item (the rest left for manual entry).
struct PresetView {
    title: String,
    description: String,
    help_text: String,
}

/// Resolve the placeholder values that can be derived from the referenced item.
fn placeholders_for(
    omission_type: OmissionType,
    reference: Uuid,
    list: Option<CandidateListId>,
    store: &CsbStore,
    locale: Locale,
) -> OmissionPlaceholders {
    match omission_type {
        OmissionType::Candidate => {
            let person = PersonId::from(reference);
            OmissionPlaceholders {
                candidate_name: store.get_person(person).map(|person| person.name.display()),
                // A candidate's position differs per list, so it can only be
                // resolved when the dialog was opened for a specific list.
                candidate_number: list
                    .and_then(|list| store.candidate_position(list, person))
                    .map(|nr| nr.to_string()),
                districts: None,
            }
        }
        OmissionType::CandidateList => OmissionPlaceholders {
            districts: store
                .get_candidate_list(CandidateListId::from(reference))
                .map(|list| list.districts_name(locale.into())),
            ..Default::default()
        },
        OmissionType::PoliticalGroup => OmissionPlaceholders::default(),
    }
}

/// The presets for this type with their descriptions interpolated.
fn preset_views(
    omission_type: OmissionType,
    reference: Uuid,
    list: Option<CandidateListId>,
    store: &CsbStore,
    locale: Locale,
) -> Vec<PresetView> {
    let placeholders = placeholders_for(omission_type, reference, list, store, locale);

    omission_type
        .presets()
        .iter()
        .map(|preset| PresetView {
            title: preset.title.clone(),
            description: placeholders.interpolate(&preset.description),
            help_text: preset.help_text.clone(),
        })
        .collect()
}

/// The page the dialog returns to: the general information page for political
/// group omissions, otherwise the political group examination overview.
fn return_path(omission_type: OmissionType, political_group: &CsbPoliticalGroup) -> String {
    match omission_type {
        OmissionType::PoliticalGroup => political_group.general_information_path().to_string(),
        _ => political_group.examination_path().to_string(),
    }
}

/// Render the "add omission" overlay dialog.
pub async fn add_omission(
    CsbAddOmissionPath {
        omission_type,
        reference,
        ..
    }: CsbAddOmissionPath,
    context: CsbContext,
    store: CsbStore,
    Query(query): Query<QueryParamState>,
    Query(OmissionListQuery { list }): Query<OmissionListQuery>,
) -> Result<Response, AppError> {
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    let presets = preset_views(
        omission_type,
        reference,
        list,
        &store,
        context.session.locale,
    );

    Ok(HtmlTemplate(
        CsbAddOmissionTemplate {
            form: FormData::new(&context.session.csrf_token),
            overlay: Overlay::new(&query),
            close_action: return_path(omission_type, &political_group),
            presets,
        },
        context,
    )
    .into_response())
}

/// Handle the submitted "add omission" form: validate, attach the category
/// derived from the path parameters, persist, and redirect back.
pub async fn add_omission_submit(
    CsbAddOmissionPath {
        omission_type,
        reference,
        ..
    }: CsbAddOmissionPath,
    context: CsbContext,
    store: CsbStore,
    Query(query): Query<QueryParamState>,
    Query(OmissionListQuery { list }): Query<OmissionListQuery>,
    Form(form): Form<OmissionForm>,
) -> Result<Response, AppError> {
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    let presets = preset_views(
        omission_type,
        reference,
        list,
        &store,
        context.session.locale,
    );

    match form.validate_create(&context.session.csrf_token) {
        Err(form_data) => Ok(HtmlTemplate(
            CsbAddOmissionTemplate {
                form: form_data,
                overlay: Overlay::new(&query),
                close_action: return_path(omission_type, &political_group),
                presets,
            },
            context,
        )
        .into_response()),
        Ok(mut omission) => {
            omission.category =
                OmissionCategory::from_type_and_reference(omission_type, reference, list);
            omission.create(&store).await?;

            Ok(query.redirect_or(return_path(omission_type, &political_group)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    use crate::{TokenValue, candidate_lists::CandidateListId, test_utils::response_body_string};

    fn sample_form(csrf_token: &TokenValue) -> OmissionForm {
        OmissionForm {
            title: "Waarborgsom ontbreekt".to_string(),
            description: "De waarborgsom ontbreekt.".to_string(),
            help_text: "Please pay the deposit.".to_string(),
            csrf_token: csrf_token.clone(),
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
        // No unresolved translation keys leaked through.
        assert!(!body.contains("[csb.omission"));
    }

    #[tokio::test]
    async fn add_candidate_list_omission_persists_the_right_category() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let list = CandidateListId::new();
        let context = CsbContext::new_test();
        let form = sample_form(&context.session.csrf_token);

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
        let form = sample_form(&context.session.csrf_token);

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
    async fn add_omission_invalid_form_rerenders() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let context = CsbContext::new_test();
        let mut form = sample_form(&context.session.csrf_token);
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
        let form = sample_form(&context.session.csrf_token);

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
            }),
            Form(form),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let omission = store.get_omission_for_test();
        assert!(matches!(
            omission.category,
            OmissionCategory::Candidate { person, list }
                if person == person_id && list == Some(list_id)
        ));
    }
}
