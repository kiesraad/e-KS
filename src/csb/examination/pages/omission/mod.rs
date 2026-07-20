use axum::{
    extract::Query,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    AppError, CsbContext, CsbStore, Form, HtmlTemplate, Overlay, QueryParamState, StreamId,
    csb::{
        OmissionCategory, OmissionType,
        examination::{
            OmissionForm,
            extractors::CsbPoliticalGroup,
            pages::{
                CsbAddOmissionPath, CsbDeleteOmissionPath, CsbOmissionOverviewPath,
                OmissionListQuery,
            },
        },
    },
    form::{FormData, ValidationError},
    structs::candidate_lists::CandidateListId,
};

mod urls;
mod views;

use urls::{add_url, overview_url, overview_url_for, return_path};
use views::{CsbAddOmissionTemplate, CsbOmissionOverviewTemplate, omission_views, preset_views};

/// The entity an omission dialog operates on, together with the list context
/// carried through its URLs and presets. Bundled so the handlers and the
/// URL/preset helpers pass one value around instead of repeating the same set
/// of fields in every signature.
#[derive(Clone, Copy)]
pub(super) struct OmissionTarget {
    pub(super) stream_id: StreamId,
    pub(super) omission_type: OmissionType,
    pub(super) reference: Uuid,
    pub(super) list: Option<CandidateListId>,
}

impl OmissionTarget {
    fn from_add_path(path: CsbAddOmissionPath, query: OmissionListQuery) -> Self {
        Self {
            stream_id: path.stream_id,
            omission_type: path.omission_type,
            reference: path.reference,
            list: query.list,
        }
    }

    fn from_overview_path(path: CsbOmissionOverviewPath, query: OmissionListQuery) -> Self {
        Self {
            stream_id: path.stream_id,
            omission_type: path.omission_type,
            reference: path.reference,
            list: query.list,
        }
    }

    /// Render the add-omission form tab. Shared by the initial GET and the
    /// re-render after an invalid submit; only the form data differs.
    fn render_add_form(
        &self,
        form: FormData<OmissionForm>,
        query: &QueryParamState,
        context: CsbContext,
        store: &CsbStore,
    ) -> Response {
        let available_districts = if self.omission_type == OmissionType::CandidateList {
            store
                .get_corrected_candidate_lists()
                .into_iter()
                .flat_map(|l| l.electoral_districts)
                .collect()
        } else {
            Vec::new()
        };
        let available_candidate_lists = if self.omission_type == OmissionType::Candidate {
            views::candidate_list_options(store, context.session.locale)
        } else {
            Vec::new()
        };
        let political_group = CsbPoliticalGroup::new_from_csb_store(store);
        HtmlTemplate(
            CsbAddOmissionTemplate {
                form,
                overlay: Overlay::new(query),
                close_action: return_path(self, &political_group),
                presets: preset_views(self, store),
                add_tab_url: add_url(self),
                overview_tab_url: overview_url(self),
                available_districts,
                available_candidate_lists,
            },
            context,
        )
        .into_response()
    }
}

/// Render the "add omission" overlay dialog.
pub async fn add_omission(
    path: CsbAddOmissionPath,
    context: CsbContext,
    store: CsbStore,
    Query(query): Query<QueryParamState>,
    Query(list_query): Query<OmissionListQuery>,
) -> Result<Response, AppError> {
    let target = OmissionTarget::from_add_path(path, list_query);
    let form = if target.omission_type == OmissionType::CandidateList {
        // Pre-fill the candidate list's paper-corrected electoral districts
        let districts = store
            .get_candidate_list(CandidateListId::from(target.reference))
            .map(|l| l.electoral_districts)
            .unwrap_or_default();
        FormData::new_with_data(OmissionForm {
            electoral_districts: districts,
            ..Default::default()
        })
    } else if target.omission_type == OmissionType::Candidate {
        // Pre-fill the list the dialog was opened from
        let candidate_lists = target.list.map(|id| vec![id]).unwrap_or_default();
        FormData::new_with_data(OmissionForm {
            candidate_lists,
            ..Default::default()
        })
    } else {
        FormData::new()
    };
    Ok(target.render_add_form(form, &query, context, &store))
}

/// Render the omissions overview page for an entity: the list of omissions
/// already added, shown in the same dialog as the add-omission form but on its
/// own tab (and its own route).
pub async fn overview(
    path: CsbOmissionOverviewPath,
    context: CsbContext,
    store: CsbStore,
    Query(query): Query<QueryParamState>,
    Query(list_query): Query<OmissionListQuery>,
) -> Result<Response, AppError> {
    let target = OmissionTarget::from_overview_path(path, list_query);
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    let overview_tab_url = overview_url(&target);

    Ok(HtmlTemplate(
        CsbOmissionOverviewTemplate {
            overlay: Overlay::new(&query),
            close_action: return_path(&target, &political_group),
            omissions: omission_views(&target, &store, &overview_tab_url)?,
            add_tab_url: add_url(&target),
            overview_tab_url,
        },
        context,
    )
    .into_response())
}

/// Handle the submitted "add omission" form: validate, attach the category
/// derived from the path parameters, persist, and redirect back.
pub async fn add_omission_submit(
    path: CsbAddOmissionPath,
    context: CsbContext,
    store: CsbStore,
    Query(query): Query<QueryParamState>,
    Query(list_query): Query<OmissionListQuery>,
    Form(form): Form<OmissionForm>,
) -> Result<Response, AppError> {
    let target = OmissionTarget::from_add_path(path, list_query);

    // For candidate list omissions at least one district muse be selected
    let districts = form.electoral_districts.clone();
    if target.omission_type == OmissionType::CandidateList && districts.is_empty() {
        let errors = vec![(
            "electoral_districts".to_string(),
            ValidationError::ChooseAtLeastOneOption,
        )];
        return Ok(target.render_add_form(
            FormData::new_with_errors(form, errors),
            &query,
            context,
            &store,
        ));
    }

    // For candidate omissions at least one list must be selected
    let candidate_lists = form.candidate_lists.clone();
    if target.omission_type == OmissionType::Candidate && candidate_lists.is_empty() {
        let errors = vec![(
            "candidate_lists".to_string(),
            ValidationError::ChooseAtLeastOneOption,
        )];
        return Ok(target.render_add_form(
            FormData::new_with_errors(form, errors),
            &query,
            context,
            &store,
        ));
    }

    match form.validate_create() {
        Err(form_data) => Ok(target.render_add_form(form_data, &query, context, &store)),
        Ok(mut omission) => {
            omission.category = if target.omission_type == OmissionType::CandidateList {
                OmissionCategory::CandidateList(districts)
            } else {
                OmissionCategory::from_type_and_reference(
                    target.omission_type,
                    target.reference,
                    candidate_lists,
                )
            };
            omission.create(&store).await?;

            let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
            Ok(query.redirect_or(return_path(&target, &political_group)))
        }
    }
}

/// Remove a single omission and return to the overview it was removed from (the
/// `redirect_to` carried by the button, falling back to the overview derived
/// from the omission's category).
pub async fn delete_omission(
    path: CsbDeleteOmissionPath,
    _context: CsbContext,
    store: CsbStore,
    Query(query): Query<QueryParamState>,
) -> Result<Response, AppError> {
    let CsbDeleteOmissionPath {
        stream_id,
        omission_id,
    } = path;
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
        csb::{Omission, OmissionType},
        structs::{candidate_lists::CandidateListId, persons::PersonId},
        test_utils::{response_body_string, sample_candidate_list},
    };

    fn sample_form() -> OmissionForm {
        OmissionForm {
            title: "Waarborgsom ontbreekt".to_string(),
            description: "De waarborgsom ontbreekt.".to_string(),
            help_text: "Please pay the deposit.".to_string(),
            recoverable: true,
            electoral_districts: Vec::new(),
            candidate_lists: Vec::new(),
        }
    }

    /// Collapse whitespace so assertions can match attributes the template
    /// renders on separate lines.
    fn normalized(body: &str) -> String {
        body.split_whitespace().collect::<Vec<_>>().join(" ")
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
    async fn add_omission_offers_and_prefills_corrected_districts() {
        use crate::test_utils::sample_candidate_list;

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let list_id = CandidateListId::new();
        // The imported list covers Utrecht; the corrections moved it to Groningen.
        store.add_candidate_list(sample_candidate_list(list_id));
        let mut corrected = sample_candidate_list(list_id);
        corrected.electoral_districts = vec![crate::ElectoralDistrict::GR];
        store.set_paper_corrected_candidate_list(corrected);

        let response = add_omission(
            CsbAddOmissionPath {
                stream_id,
                omission_type: OmissionType::CandidateList,
                reference: list_id.into(),
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
        let body = normalized(&response_body_string(response).await);
        // The corrected district is selectable and pre-filled, the imported
        // one is disabled.
        assert!(body.contains(r#"data-district-nl="Groningen" checked />"#));
        assert!(body.contains(r#"data-district-nl="Utrecht" disabled />"#));
    }

    #[tokio::test]
    async fn add_omission_offers_districts_of_paper_added_lists() {
        use crate::test_utils::sample_candidate_list;

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let list_id = CandidateListId::new();
        // The list only exists in the corrected projection (added on paper).
        store.set_paper_corrected_candidate_list(sample_candidate_list(list_id));

        let response = add_omission(
            CsbAddOmissionPath {
                stream_id,
                omission_type: OmissionType::CandidateList,
                reference: list_id.into(),
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
        let body = normalized(&response_body_string(response).await);
        assert!(body.contains(r#"data-district-nl="Utrecht" checked />"#));
    }

    #[tokio::test]
    async fn candidate_dialog_offers_paper_added_lists() {
        use crate::test_utils::{sample_candidate_list, sample_person};

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let person = sample_person(PersonId::new());
        let person_id = person.id;
        store.add_person(person);
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.candidates = vec![person_id];
        store.set_paper_corrected_candidate_list(list);

        let response = add_omission(
            CsbAddOmissionPath {
                stream_id,
                omission_type: OmissionType::Candidate,
                reference: person_id.into(),
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
        // The paper-added list is a selectable option.
        assert!(body.contains(&format!("omission_candidate_list_{list_id}")));
    }

    #[tokio::test]
    async fn candidate_dialog_resolves_placeholders_for_paper_added_candidates() {
        use crate::test_utils::{sample_candidate_list, sample_person};

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        // Both the candidate and the list only exist in the corrected
        // projection: they were added during paper corrections.
        let person = sample_person(PersonId::new());
        let person_id = person.id;
        store
            .data
            .write()
            .paper_corrected_data
            .persons
            .insert(person_id, person);
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.candidates = vec![person_id];
        store.set_paper_corrected_candidate_list(list);

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
        // The name and position placeholders resolve through the corrected
        // projection.
        assert!(body.contains("Kandidaat nr. 1, Jansen, H.A.H.A. (Henk)"));
    }

    #[tokio::test]
    async fn overview_tab_lists_added_omissions_with_details() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let list = CandidateListId::new();

        // Store the list so get_candidate_list_omissions can look up its districts.
        store.add_candidate_list(sample_candidate_list(list));

        // A recoverable and an irreparable omission covering the list's district.
        Omission::new(
            OmissionCategory::CandidateList(vec![crate::ElectoralDistrict::UT]),
            "Waarborgsom ontbreekt".to_string(),
            "De waarborgsom ontbreekt.".to_string(),
            "Betaal de waarborgsom.".to_string(),
        )
        .create(&store)
        .await
        .unwrap();
        let mut irreparable = Omission::new(
            OmissionCategory::CandidateList(vec![crate::ElectoralDistrict::UT]),
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
        // The sidebar still links back to the add-omission form, marked as an
        // in-overlay navigation.
        assert!(body.contains("steps-nav"));
        assert!(body.contains(&format!(
            "/csb/examination/{stream_id}/omission/candidate-list/{list}?&#38;overlay=true\""
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
        store.add_candidate_list(sample_candidate_list(list));

        let omission = Omission::new(
            OmissionCategory::CandidateList(vec![crate::ElectoralDistrict::UT]),
            "Waarborgsom ontbreekt".to_string(),
            "De waarborgsom ontbreekt.".to_string(),
            String::new(),
        );
        omission.create(&store).await.unwrap();
        let omission_id = omission.id;
        assert_eq!(store.get_candidate_list_omissions(list).unwrap().len(), 1);

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
        assert!(store.get_candidate_list_omissions(list).unwrap().is_empty());
        // ...and without an explicit redirect we fall back to the political group overview
        let location = response
            .headers()
            .get("Location")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.contains(&format!(
            "/csb/examination/{stream_id}/omission/political-group/{stream_id}/overview"
        )));
    }

    #[tokio::test]
    async fn delete_omission_honours_the_redirect_to() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let omission = Omission::new(
            OmissionCategory::PoliticalGroup,
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
        assert!(store.get_political_group_omissions().is_empty());
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
    async fn add_candidate_list_omission_persists_category() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let list = CandidateListId::new();
        let context = CsbContext::new_test();
        let form = OmissionForm {
            electoral_districts: vec![crate::ElectoralDistrict::GR, crate::ElectoralDistrict::DR],
            ..sample_form()
        };

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
            &omission.category,
            OmissionCategory::CandidateList(districts)
                if districts == &[crate::ElectoralDistrict::GR, crate::ElectoralDistrict::DR]
        ));
    }

    #[tokio::test]
    async fn add_candidate_list_omission_without_districts_rerenders_form() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let list = CandidateListId::new();
        let context = CsbContext::new_test();
        // No districts selected: should re-render the form with an error
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

        assert_eq!(response.status(), StatusCode::OK);
        assert!(store.get_political_group_omissions().is_empty());
    }

    #[tokio::test]
    async fn add_political_group_omission_persists_category() {
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

        assert_eq!(store.get_political_group_omissions().len(), 1);
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
        assert!(store.get_political_group_omissions().is_empty());
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
        store.add_person(person);
        store.add_candidate_list(list);

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
        // Both former "candidate" and "person" presets are shown.
        assert!(body.contains("Kopie ID ontbreekt"));
        // ...while a preset scoped to the listing on the list does not.
        assert!(!body.contains("onjuiste nadere aanduidingen"));
    }

    #[tokio::test]
    async fn candidate_position_is_scoped_to_the_referenced_list() {
        use crate::test_utils::{sample_candidate_list, sample_person};

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        // The same candidate sits at different positions on two lists.
        let person = sample_person(PersonId::new());
        let person_id = person.id;
        store.add_person(person);

        let first_list_id = CandidateListId::new();
        let mut first_list = sample_candidate_list(first_list_id);
        first_list.candidates = vec![person_id];
        store.add_candidate_list(first_list);

        let second_list_id = CandidateListId::new();
        let mut second_list = sample_candidate_list(second_list_id);
        second_list.candidates = vec![PersonId::new(), person_id];
        store.add_candidate_list(second_list);

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
    async fn add_candidate_omission_persists_the_selected_lists() {
        use crate::test_utils::{sample_candidate_list, sample_person};

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let context = CsbContext::new_test();

        let person = sample_person(PersonId::new());
        let person_id = person.id;
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.candidates = vec![person_id];
        store.add_person(person);
        store.add_candidate_list(list);

        let form = OmissionForm {
            candidate_lists: vec![list_id],
            ..sample_form()
        };

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
        // The dialog redirects back to the candidate detail page it was opened from.
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
            OmissionCategory::Candidate { person, ref lists }
                if person == person_id && lists == &[list_id]
        ));
    }

    #[tokio::test]
    async fn add_candidate_omission_without_lists_rerenders_form() {
        use crate::test_utils::{sample_candidate_list, sample_person};

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let context = CsbContext::new_test();

        let person = sample_person(PersonId::new());
        let person_id = person.id;
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.candidates = vec![person_id];
        store.add_person(person);
        store.add_candidate_list(list);

        // No lists selected: should re-render the form with an error
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
            Form(sample_form()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(store.get_omission_count(), 0);
    }
}
