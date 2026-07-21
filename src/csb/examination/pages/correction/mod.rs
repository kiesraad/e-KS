use askama::Template;
use axum::{
    extract::Query,
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::{
    AppError, Context, CsbContext, CsbStore, Form, HtmlTemplate, Overlay, QueryParamState,
    candidate_lists::CandidateListId,
    common::{DateOfBirth, DisplayName, Initials, LastName, PlaceOfResidence},
    csb::examination::{
        extractors::CsbPoliticalGroup,
        pages::{
            CandidateCorrectionField, CsbDisplayNameCorrectionPath, CsbPersonCorrectionPath,
            OmissionListQuery,
        },
    },
    filters,
    form::{FormData, ValidationError},
    persons::{Person, PersonId},
    structs::csb::{Correction, PersonCorrection},
};

/// Form backing the correction overlay. A single free-text value is submitted
/// and validated as the appropriate typed value in the handler.
#[derive(Deserialize, Debug, Default)]
pub struct CorrectionForm {
    pub value: String,
}

#[derive(Template)]
#[template(path = "csb/examination/pages/correction.html")]
struct CsbCorrectionTemplate {
    overlay: Overlay,
    close_action: String,
    /// Translated label for the field being corrected.
    label: String,
    /// The value from the original imported data.
    imported_value: String,
    /// The paper-corrected value, shown only when it differs from the imported.
    paper_corrected_value: Option<String>,
    form: FormData<CorrectionForm>,
}

/// Render the correction overlay for the political-group display name.
pub async fn display_name_correction(
    _: CsbDisplayNameCorrectionPath,
    context: CsbContext,
    store: CsbStore,
    Query(query): Query<QueryParamState>,
) -> Result<Response, AppError> {
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    let corrected_store = store.paper_corrected();
    let locale = context.session.locale;

    let imported_value = store
        .get_imported_political_group()
        .csb_display_name(store.first_imported_candidate_name().as_ref());
    let paper_corrected_value = {
        let corrected = corrected_store
            .get_political_group()
            .csb_display_name(corrected_store.first_candidate_name().as_ref());
        if corrected != imported_value {
            Some(corrected)
        } else {
            None
        }
    };
    let prefill = store
        .get_corrected_display_name()
        .map(|dn| dn.to_string())
        .unwrap_or_else(|| {
            paper_corrected_value
                .clone()
                .unwrap_or_else(|| imported_value.clone())
        });

    Ok(render_correction(
        context,
        query,
        political_group.general_information_path().to_string(),
        crate::trans!("political_group.display_name", locale),
        imported_value,
        paper_corrected_value,
        FormData::new_with_data(CorrectionForm { value: prefill }),
    ))
}

/// Handle the submitted display-name correction form.
pub async fn display_name_correction_submit(
    _: CsbDisplayNameCorrectionPath,
    context: CsbContext,
    store: CsbStore,
    Query(query): Query<QueryParamState>,
    Form(form): Form<CorrectionForm>,
) -> Result<Response, AppError> {
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    let close_action = political_group.general_information_path().to_string();
    let locale = context.session.locale;

    match form.value.parse::<DisplayName>() {
        Err(err) => {
            let corrected_store = store.paper_corrected();
            let imported_value = store
                .get_imported_political_group()
                .csb_display_name(store.first_imported_candidate_name().as_ref());
            let paper_corrected_value = {
                let corrected = corrected_store
                    .get_political_group()
                    .csb_display_name(corrected_store.first_candidate_name().as_ref());
                if corrected != imported_value {
                    Some(corrected)
                } else {
                    None
                }
            };
            Ok(render_correction(
                context,
                query,
                close_action,
                crate::trans!("political_group.display_name", locale),
                imported_value,
                paper_corrected_value,
                FormData::new_with_errors(form, vec![("value".to_string(), err)]),
            ))
        }
        Ok(display_name) => {
            store
                .update(crate::CsbEvent::UpdateCorrection(Correction::DisplayName(
                    display_name,
                )))
                .await?;
            Ok(query.redirect_or(close_action))
        }
    }
}

/// Render the correction overlay for a specific personal-data field of a candidate.
pub async fn person_correction(
    path: CsbPersonCorrectionPath,
    context: CsbContext,
    store: CsbStore,
    Query(query): Query<QueryParamState>,
    Query(list_query): Query<OmissionListQuery>,
) -> Result<Response, AppError> {
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    let close_action = return_path(&political_group, path.person_id, list_query.list);
    let locale = context.session.locale;

    let imported = store.get_imported_person(path.person_id);
    let paper_corrected = store.paper_corrected().get_person(path.person_id).ok();
    let ex_officio = store.get_corrected_person(path.person_id);

    let field_values = FieldValues::build(
        path.field,
        imported.as_ref(),
        paper_corrected.as_ref(),
        ex_officio.as_ref(),
    );

    let prefill = field_values.current_correction.unwrap_or_else(|| {
        field_values
            .paper_corrected
            .clone()
            .unwrap_or_else(|| field_values.imported.clone())
    });

    Ok(render_correction(
        context,
        query,
        close_action,
        field_label(path.field, locale),
        field_values.imported,
        field_values.paper_corrected,
        FormData::new_with_data(CorrectionForm { value: prefill }),
    ))
}

/// Handle the submitted person-field correction form.
pub async fn person_correction_submit(
    path: CsbPersonCorrectionPath,
    context: CsbContext,
    store: CsbStore,
    Query(query): Query<QueryParamState>,
    Query(list_query): Query<OmissionListQuery>,
    Form(form): Form<CorrectionForm>,
) -> Result<Response, AppError> {
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    let close_action = return_path(&political_group, path.person_id, list_query.list);
    let locale = context.session.locale;

    let correction = parse_person_correction(path.field, &form.value);

    match correction {
        Err(err) => {
            let imported = store.get_imported_person(path.person_id);
            let paper_corrected = store.paper_corrected().get_person(path.person_id).ok();
            let ex_officio = store.get_corrected_person(path.person_id);
            let field_values = FieldValues::build(
                path.field,
                imported.as_ref(),
                paper_corrected.as_ref(),
                ex_officio.as_ref(),
            );
            Ok(render_correction(
                context,
                query,
                close_action,
                field_label(path.field, locale),
                field_values.imported,
                field_values.paper_corrected,
                FormData::new_with_errors(form, vec![("value".to_string(), err)]),
            ))
        }
        Ok(person_correction) => {
            store
                .update(crate::CsbEvent::UpdateCorrection(Correction::Person(
                    path.person_id,
                    person_correction,
                )))
                .await?;
            Ok(query.redirect_or(close_action))
        }
    }
}

/// The return path after closing or saving in the correction overlay:
/// the candidate detail page when a list is known, otherwise the examination
/// overview for the political group.
fn return_path(
    political_group: &CsbPoliticalGroup,
    person_id: PersonId,
    list: Option<CandidateListId>,
) -> String {
    match list {
        Some(list_id) => political_group
            .candidate_path(&list_id, &person_id)
            .to_string(),
        None => political_group.examination_path().to_string(),
    }
}

/// Translated label for a correction field, reusing the existing person-field
/// translation keys.
fn field_label(field: CandidateCorrectionField, locale: crate::Locale) -> String {
    match field {
        CandidateCorrectionField::Initials => {
            crate::trans!("person.fields.initials", locale)
        }
        CandidateCorrectionField::LastName => {
            crate::trans!("person.fields.last_name", locale)
        }
        CandidateCorrectionField::DateOfBirth => {
            crate::trans!("person.fields.date_of_birth", locale)
        }
        CandidateCorrectionField::PlaceOfResidence => {
            crate::trans!("person.fields.place_of_residence", locale)
        }
    }
}

/// Parse the submitted form value into the appropriate `PersonCorrection`
/// variant for the given field.
fn parse_person_correction(
    field: CandidateCorrectionField,
    value: &str,
) -> Result<PersonCorrection, ValidationError> {
    match field {
        CandidateCorrectionField::Initials => {
            value.parse::<Initials>().map(PersonCorrection::Initials)
        }
        CandidateCorrectionField::LastName => {
            value.parse::<LastName>().map(PersonCorrection::LastName)
        }
        CandidateCorrectionField::DateOfBirth => value
            .parse::<DateOfBirth>()
            .map(PersonCorrection::DateOfBirth),
        CandidateCorrectionField::PlaceOfResidence => value
            .parse::<PlaceOfResidence>()
            .map(PersonCorrection::PlaceOfResidence),
    }
}

/// The three value strings needed for the correction overlay, extracted from
/// the three data projections.
struct FieldValues {
    imported: String,
    paper_corrected: Option<String>,
    current_correction: Option<String>,
}

impl FieldValues {
    fn build(
        field: CandidateCorrectionField,
        imported: Option<&Person>,
        paper_corrected: Option<&Person>,
        ex_officio: Option<&Person>,
    ) -> Self {
        let imported_val = imported
            .map(|p| extract_field(field, p))
            .unwrap_or_default();

        let paper_corrected_val = paper_corrected
            .map(|p| extract_field(field, p))
            .filter(|v| v != &imported_val)
            .filter(|v| !v.is_empty());

        let current_correction = ex_officio.and_then(|p| {
            let corrected_val = extract_field(field, p);
            if corrected_val.is_empty() {
                None
            } else {
                Some(corrected_val)
            }
        });

        Self {
            imported: imported_val,
            paper_corrected: paper_corrected_val,
            current_correction,
        }
    }
}

/// Extract the string representation of a specific field from a person,
/// using the same formatting as the examination page display.
fn extract_field(field: CandidateCorrectionField, person: &Person) -> String {
    match field {
        CandidateCorrectionField::Initials => person.name.initials.to_string(),
        CandidateCorrectionField::LastName => person.name.last_name_with_prefix(),
        CandidateCorrectionField::DateOfBirth => {
            DateOfBirth::format_option(&person.personal_data.date_of_birth)
        }
        CandidateCorrectionField::PlaceOfResidence => person
            .personal_data
            .place_of_residence
            .as_ref()
            .map(|p| p.to_string())
            .unwrap_or_default(),
    }
}

fn render_correction(
    context: CsbContext,
    query: QueryParamState,
    close_action: String,
    label: String,
    imported_value: String,
    paper_corrected_value: Option<String>,
    form: FormData<CorrectionForm>,
) -> Response {
    HtmlTemplate(
        CsbCorrectionTemplate {
            overlay: Overlay::new(&query),
            close_action,
            label,
            imported_value,
            paper_corrected_value,
            form,
        },
        context,
    )
    .into_response()
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
    async fn display_name_correction_renders_overlay() {
        use crate::test_utils::sample_political_group;

        let store = crate::CsbStore::new_for_test();
        let stream_id = store.stream_id;
        store.set_political_group(sample_political_group());

        let response = display_name_correction(
            CsbDisplayNameCorrectionPath { stream_id },
            CsbContext::new_test(),
            store,
            Query(QueryParamState::default()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("name=\"value\""));
        assert!(body.contains("name=\"csrf_token\""));
    }

    #[tokio::test]
    async fn display_name_correction_submit_persists_correction() {
        use crate::test_utils::sample_political_group;

        let store = crate::CsbStore::new_for_test();
        let stream_id = store.stream_id;
        store.set_political_group(sample_political_group());

        let response = display_name_correction_submit(
            CsbDisplayNameCorrectionPath { stream_id },
            CsbContext::new_test(),
            store.clone(),
            Query(QueryParamState::default()),
            Form(CorrectionForm {
                value: "Nieuwe Naam".to_string(),
            }),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            store.get_corrected_display_name().map(|d| d.to_string()),
            Some("Nieuwe Naam".to_string())
        );
    }

    #[tokio::test]
    async fn display_name_correction_submit_rerenders_on_invalid_value() {
        let store = crate::CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let response = display_name_correction_submit(
            CsbDisplayNameCorrectionPath { stream_id },
            CsbContext::new_test(),
            store.clone(),
            Query(QueryParamState::default()),
            Form(CorrectionForm {
                value: String::new(),
            }),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(store.get_corrected_display_name().is_none());
    }

    #[tokio::test]
    async fn person_correction_renders_overlay_with_initials() {
        let store = crate::CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let person = sample_person(PersonId::new());
        let person_id = person.id;
        let list_id = crate::candidate_lists::CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.candidates = vec![person_id];
        store.add_person(person);
        store.add_candidate_list(list);

        let response = person_correction(
            CsbPersonCorrectionPath {
                stream_id,
                person_id,
                field: CandidateCorrectionField::Initials,
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
        assert!(body.contains("name=\"value\""));
        // Sample person has initials H.A.H.A.
        assert!(body.contains("H.A.H.A."));
    }

    #[tokio::test]
    async fn person_correction_submit_persists_initials() {
        let store = crate::CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let person = sample_person(PersonId::new());
        let person_id = person.id;
        store.add_person(person);

        let response = person_correction_submit(
            CsbPersonCorrectionPath {
                stream_id,
                person_id,
                field: CandidateCorrectionField::Initials,
            },
            CsbContext::new_test(),
            store.clone(),
            Query(QueryParamState::default()),
            Query(OmissionListQuery::default()),
            Form(CorrectionForm {
                value: "X.Y.Z.".to_string(),
            }),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let corrected = store.get_corrected_person(person_id).unwrap();
        assert_eq!(corrected.name.initials.to_string(), "X.Y.Z.");
    }

    #[tokio::test]
    async fn person_correction_submit_rerenders_on_invalid_initials() {
        let store = crate::CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let person = sample_person(PersonId::new());
        let person_id = person.id;
        store.add_person(person);

        let response = person_correction_submit(
            CsbPersonCorrectionPath {
                stream_id,
                person_id,
                field: CandidateCorrectionField::Initials,
            },
            CsbContext::new_test(),
            store.clone(),
            Query(QueryParamState::default()),
            Query(OmissionListQuery::default()),
            Form(CorrectionForm {
                value: String::new(),
            }),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(store.get_corrected_person(person_id).is_none());
    }
}
